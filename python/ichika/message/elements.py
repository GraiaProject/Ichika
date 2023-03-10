from __future__ import annotations

import base64
import pathlib
from dataclasses import dataclass
from enum import Enum
from functools import total_ordering
from io import BytesIO
from typing import Generic, Literal, Optional
from typing_extensions import Self, TypeAlias, TypeGuard, TypeVar

import aiohttp
from graia.amnesia.message import Element
from graia.amnesia.message.element import Text as Text

from .. import core
from ._sealed import SealedAudio, SealedImage, SealedMarketFace


@dataclass
class At(Element):
    target: int
    display: str | None = None

    def __str__(self) -> str:
        return f"@{self.target}"


class AtAll(Element):
    def __str__(self) -> str:
        return "@全体成员"

    def __repr__(self) -> str:
        return "AtAll()"


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

    def __repr__(self) -> str:
        return f"FingerGuessing(choice={self.choice})"


DiceValues: TypeAlias = Literal[1, 2, 3, 4, 5, 6]


class Dice(Element):
    value: DiceValues

    def __init__(self, value: DiceValues) -> None:
        if value not in range(1, 6 + 1):
            raise ValueError(f"{value} 不是有效的骰子值")
        self.value = value

    def __str__(self) -> str:
        return f"[骰子: {self.value}]"

    def __repr__(self) -> str:
        return f"Dice(value={self.value})"


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

    def __repr__(self) -> str:
        return f"Face(index={self.index}, name={self.name})"


@dataclass
class LightApp(Element):
    content: str

    def __str__(self) -> str:
        return "[小程序]"


T_Audio = TypeVar("T_Audio", bound=Optional[SealedAudio], default=SealedAudio)


class Audio(Generic[T_Audio], Element):
    url: str
    raw: T_Audio
    _data_cache: bytes | None

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


class Image(Generic[T_Image], Element):
    url: str
    raw: T_Image
    _data_cache: bytes | None

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


TYPE_MAP = {
    cls.__name__: cls
    for cls in (Text, At, AtAll, FingerGuessing, Dice, Face, LightApp, Audio, Image, FlashImage, MarketFace, Audio)
}
