from __future__ import annotations

import base64
import pathlib
import re
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from functools import total_ordering
from io import BytesIO
from typing import TYPE_CHECKING, Callable, Generic, Literal, Optional
from typing_extensions import Self, TypeAlias, TypeGuard, TypeVar

import aiohttp
from graia.amnesia.message import Element, MessageChain
from graia.amnesia.message.element import Text as Text

from .. import core
from ._sealed import SealedAudio, SealedImage, SealedMarketFace

if TYPE_CHECKING:
    from ..client import Client as __Client


@dataclass
class Reply(Element):
    seq: int
    sender: int
    time: datetime
    content: str


@dataclass
class At(Element):
    target: int
    display: str | None = None

    def __str__(self) -> str:
        return f"@{self.target}"


@dataclass
class AtAll(Element):
    def __str__(self) -> str:
        return "@全体成员"


@dataclass(init=False)
class FingerGuessing(Element):
    @total_ordering
    class Choice(Enum):
        Rock = "石头"
        Scissors = "剪刀"
        Paper = "布"

        def __eq__(self, other: Self) -> bool:
            if not isinstance(other, FingerGuessing.Choice):
                raise TypeError(f"{other} 不是 FingerGuessing.Choice")
            return self.value == other.value

        def __lt__(self, other: Self) -> bool:
            if not isinstance(other, FingerGuessing.Choice):
                raise TypeError(f"{other} 不是 FingerGuessing.Choice")
            return (self.name, other.name) in {
                ("Rock", "Scissors"),
                ("Scissors", "Paper"),
                ("Paper", "Rock"),
            }

    choice: Choice

    def __init__(
        self,
        choice: Literal["Rock", "Paper", "Scissors" "石头", "剪刀", "布"] | Choice,
    ) -> None:
        C = FingerGuessing.Choice
        if isinstance(choice, str):
            self.choice = C[choice] if choice in C else C(choice)
        if isinstance(choice, C):
            self.choice = choice
        raise TypeError(f"无效的猜拳参数：{choice}")

    def __str__(self) -> str:
        return f"[猜拳: {self.choice.value}]"


DiceValues: TypeAlias = Literal[1, 2, 3, 4, 5, 6]


@dataclass
class Dice(Element):
    value: Literal[1, 2, 3, 4, 5, 6]

    def __str__(self) -> str:
        return f"[骰子: {self.value}]"


@dataclass(init=False)
class Face(Element):
    def __init__(self, index: int, name: str | None = None) -> None:
        self.index = index
        self.name = name or core.face_name_from_id(index)

    @classmethod
    def from_name(cls, name: str) -> Self:
        index = core.face_id_from_name(name)
        if index is None:
            raise ValueError("未知表情")
        return cls(index, name)

    def __str__(self) -> str:
        return f"[表情: {self.name}]"


@dataclass
class MusicShare(Element):
    """音乐分享

    音乐分享本质为 “小程序”
    但是可以用不同方式发送
    并且风控几率较小
    """

    kind: Literal["QQ", "Netease", "Migu", "Kugou", "Kuwo"]
    title: str
    summary: str
    jump_url: str
    picture_url: str
    music_url: str
    brief: str

    def __str__(self) -> str:
        return f"[{self.kind}音乐分享: {self.title}]"


@dataclass
class LightApp(Element):
    """小程序

    本框架不辅助音乐分享外的小程序构造与发送
    """

    content: str
    """JSON 内容"""

    def __str__(self) -> str:
        return "[小程序]"


@dataclass
class ForwardCard(Element):
    """未下载的合并转发消息，本质为 XML 卡片"""

    res_id: str
    file_name: str
    content: str

    def __str__(self) -> str:
        return "[合并转发]"

    async def download(self, client: __Client) -> list[ForwardMessage]:
        """使用 aiohttp 下载本转发卡片对应的转发消息"""

        async def _downloader(method: Literal["get", "post"], url: str, headers: dict[str, str], body: bytes) -> bytes:
            async with aiohttp.ClientSession(headers=headers) as session:
                async with session.request(method, url, data=body) as resp:
                    return await resp.read()

        return await client.download_forward_msg(_downloader, self.res_id)


@dataclass
class ForwardMessage:
    """已下载的合并转发消息"""

    sender_id: int
    time: datetime
    sender_name: str
    content: MessageChain | list[ForwardMessage]


@dataclass
class RichMessage(Element):
    """卡片消息"""

    service_id: int
    """服务 ID"""
    content: str
    """卡片内容"""

    def __str__(self) -> str:
        return "[富文本卡片]"


T_Audio = TypeVar("T_Audio", bound=Optional[SealedAudio], default=SealedAudio)


@dataclass(init=False)
class Audio(Generic[T_Audio], Element):
    url: str
    raw: T_Audio = field(compare=False)
    _data_cache: bytes | None = field(repr=False, compare=False)

    def __init__(self, url: str, raw: T_Audio = None) -> None:
        self.url = url
        self._data_cache = None
        self.raw = raw

    @classmethod
    def build(cls, data: bytes | BytesIO | pathlib.Path) -> Audio[None]:
        if isinstance(data, BytesIO):
            data = data.read()
        elif isinstance(data, pathlib.Path):
            data = data.read_bytes()
        audio = Audio(f"base64://{base64.urlsafe_b64encode(data)}")
        audio._data_cache = data
        return audio

    @classmethod
    def _check(cls, elem: Element) -> TypeGuard[Audio[Optional[SealedAudio]]]:
        return isinstance(elem, Audio)

    @property
    def md5(self: Audio[SealedAudio]) -> bytes:
        return self.raw.md5

    @property
    def size(self: Audio[SealedAudio]) -> int:
        return self.raw.size

    @property
    def file_type(self: Audio[SealedAudio]) -> int:
        return self.raw.file_type

    async def fetch(self) -> bytes:
        if self._data_cache is None:
            if self.url.startswith("base64://"):
                self._data_cache = base64.urlsafe_b64decode(self.url[8:])
            else:
                async with aiohttp.ClientSession() as session:
                    async with session.get(self.url) as resp:
                        self._data_cache = await resp.read()
        return self._data_cache

    def __repr__(self) -> str:
        return "[音频]"


T_Image = TypeVar("T_Image", bound=Optional[SealedImage], default=SealedImage)


@dataclass(init=False)
class Image(Generic[T_Image], Element):
    url: str
    raw: T_Image = field(compare=False)
    _data_cache: bytes | None = field(repr=False, compare=False)

    def __init__(self, url: str, raw: T_Image = None) -> None:
        self.url = url
        self._data_cache = None
        self.raw = raw

    @classmethod
    def build(cls, data: bytes | BytesIO | pathlib.Path) -> Image[None]:
        if isinstance(data, BytesIO):
            data = data.read()
        elif isinstance(data, pathlib.Path):
            data = data.read_bytes()
        img = Image(f"base64://{base64.urlsafe_b64encode(data)}")
        img._data_cache = data
        return img

    @classmethod
    def _check(cls, elem: Element) -> TypeGuard[Image[Optional[SealedImage]]]:
        return isinstance(elem, Image)

    @property
    def md5(self: Image[SealedImage]) -> bytes:
        return self.raw.md5

    @property
    def size(self: Image[SealedImage]) -> int:
        return self.raw.size

    @property
    def width(self: Image[SealedImage]) -> int:
        return self.raw.width

    @property
    def height(self: Image[SealedImage]) -> int:
        return self.raw.height

    @property
    def image_type(self: Image[SealedImage]) -> int:
        return self.raw.image_type

    async def fetch(self) -> bytes:
        if self._data_cache is None:
            if self.url.startswith("base64://"):
                self._data_cache = base64.urlsafe_b64decode(self.url[8:])
            else:
                async with aiohttp.ClientSession() as session:
                    async with session.get(self.url) as resp:
                        self._data_cache = await resp.read()
        return self._data_cache

    def as_flash(self) -> FlashImage[T_Image]:
        img = FlashImage(self.url, self.raw)
        img._data_cache = self._data_cache
        return img

    def __str__(self) -> str:
        return "[图片]"


@dataclass(init=False)
class FlashImage(Image[T_Image]):
    @classmethod
    def build(cls, data: bytes | BytesIO | pathlib.Path) -> FlashImage[None]:
        return Image.build(data).as_flash()

    @classmethod
    def _check(cls, elem: Element) -> TypeGuard[FlashImage[Optional[SealedImage]]]:
        return isinstance(elem, FlashImage)

    def as_image(self) -> Image[T_Image]:
        img = Image(self.url, self.raw)
        img._data_cache = self._data_cache
        return img

    def __str__(self) -> str:
        return "[闪照]"


class Video(Element):
    ...


class MarketFace(Element):
    def __init__(self, raw: SealedMarketFace) -> None:
        self.raw = raw

    @property
    def name(self) -> str:
        return self.raw.name

    def __str__(self) -> str:
        return f"[商城表情:{self.name}]"

    def __repr__(self) -> str:
        return f"MarketFace(name={self.name})"


_DESERIALIZE_INV: dict[str, Callable[..., Element]] = {
    cls.__name__: cls for cls in Element.__subclasses__() if cls.__module__.startswith(("ichika", "graia.amnesia"))
}

__MUSIC_SHARE_APPID_MAP: dict[int, Literal["QQ", "Netease", "Migu", "Kugou", "Kuwo"]] = {
    100497308: "QQ",
    100495085: "Netease",
    1101053067: "Migu",
    205141: "Kugou",
    100243533: "Kuwo",
}


def _light_app_deserializer(**data) -> Element:
    import json
    from contextlib import suppress

    with suppress(ValueError, KeyError):
        # MusicShare resolver
        # https://github.com/mamoe/mirai/blob/893fb3e9f653623056f9c4bff73b4dac957cd2a2/mirai-core/src/commonMain/kotlin/message/data/lightApp.kt
        app_data = json.loads(data["content"])
        music_info = app_data["meta"]["music"]
        return MusicShare(
            kind=__MUSIC_SHARE_APPID_MAP[app_data["extra"]["appid"]],
            title=music_info["title"],
            summary=music_info["desc"],
            jump_url=music_info["jumpUrl"],
            picture_url=music_info["preview"],
            music_url=music_info["musicUrl"],
            brief=data["prompt"],
        )

    return LightApp(content=data["content"])


__RES_ID_PAT = re.compile(r"m_resid=\"(.*?)\"")
__FILE_NAME_PAT = re.compile(r"m_fileName=\"(.*?)\"")


def _rich_msg_deserializer(**data) -> Element:
    service_id: int = data["service_id"]
    content: str = data["content"]

    if (res_id_match := __RES_ID_PAT.search(content)) and (file_name_match := __FILE_NAME_PAT.search(content)):
        return ForwardCard(res_id=res_id_match[1], file_name=file_name_match[1], content=content)

    return RichMessage(service_id=service_id, content=content)


_DESERIALIZE_INV["LightApp"] = _light_app_deserializer
_DESERIALIZE_INV["RichMessage"] = _rich_msg_deserializer
