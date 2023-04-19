"""基于 `ichika.core.PlumbingClient` 封装的高层 API"""
from __future__ import annotations

from typing import Any, Awaitable, Callable, Literal, Protocol

from graia.amnesia.message import Element, MessageChain

from .core import PlumbingClient, RawMessageReceipt
from .message import _serialize_message as _serialize_msg
from .message.elements import (
    At,
    AtAll,
    Audio,
    Face,
    FlashImage,
    ForwardCard,
    ForwardMessage,
    Image,
    MusicShare,
    Reply,
    Text,
)


class HttpClientProto(Protocol):
    def __call__(
        self, method: Literal["get", "post", "delete"], url: str, headers: dict[str, str], body: bytes
    ) -> Awaitable[bytes]:
        ...


class Client(PlumbingClient):
    async def upload_friend_image(self, uin: int, data: bytes) -> Image:
        image_dict = await super().upload_friend_image(uin, data)
        image_dict.pop("type")
        return Image(**image_dict)

    async def upload_friend_audio(self, uin: int, data: bytes) -> Audio:
        audio_dict = await super().upload_friend_audio(uin, data)
        audio_dict.pop("type")
        return Audio(**audio_dict)

    async def upload_group_image(self, uin: int, data: bytes) -> Image:
        image_dict = await super().upload_group_image(uin, data)
        image_dict.pop("type")
        return Image(**image_dict)

    async def upload_group_audio(self, uin: int, data: bytes) -> Audio:
        audio_dict = await super().upload_group_audio(uin, data)
        audio_dict.pop("type")
        return Audio(**audio_dict)

    @classmethod
    def _parse_downloaded_fwd(cls, content: dict) -> ForwardMessage:
        if content.pop("type") == "Forward":
            content["content"] = cls._parse_downloaded_fwd(content.pop("content"))
        return ForwardMessage(**content)

    async def download_forward_msg(self, downloader: HttpClientProto, res_id: str) -> list[ForwardMessage]:
        origin = await super().download_forward_msg(downloader, res_id)
        return [self._parse_downloaded_fwd(content) for content in origin]

    @staticmethod
    def _validate_chain(chain: MessageChain) -> MessageChain | Element:
        if not chain:
            raise ValueError("无法发送空消息！")
        if any(not isinstance(elem, (Reply, At, AtAll, Text, Image, Face)) for elem in chain):
            if len(chain) > 1:
                raise ValueError("消息内混合了富文本和非富文本型消息！")
            elem = chain[0]
            if isinstance(elem, (Audio, MusicShare)):
                return chain[0]
        return chain

    @staticmethod
    async def _validate_mm(uin: int, elem: Element, uploader: Callable[[int, bytes], Awaitable[Image]]) -> Element:
        if Image._check(elem) and elem.raw is None:
            new_img = await uploader(uin, await elem.fetch())
            if FlashImage._check(elem):
                new_img = new_img.as_flash()
            return new_img
        return elem

    async def _prepare_forward(self, uin: int, fwd: ForwardMessage) -> dict[str, Any]:
        data = {
            "sender_id": fwd.sender_id,
            "sender_name": fwd.sender_name,
            "time": int(fwd.time.timestamp()),
        }
        if isinstance(fwd.content, MessageChain):
            data["type"] = "Message"
            if isinstance(self._validate_chain(fwd.content), Audio):
                raise TypeError(f"转发消息不允许使用音频: {fwd.content:r}")
            content = MessageChain(
                [await self._validate_mm(uin, elem, self.upload_group_image) for elem in fwd.content]
            )
            data["content"] = _serialize_msg(content)
        else:
            data["type"] = "Forward"
            data["content"] = [await self._prepare_forward(uin, f) for f in fwd.content]
        return data

    async def upload_forward_msg(self, group_uin: int, msgs: list[ForwardMessage]) -> ForwardCard:
        res_id, file_name, content = await super().upload_forward_msg(
            group_uin, [await self._prepare_forward(group_uin, msg) for msg in msgs]
        )
        return ForwardCard(res_id, file_name, content)

    async def _send_special_element(self, uin: int, kind: str, element: Element) -> RawMessageReceipt:
        if Audio._check(element):
            if element.raw is None:
                uploader = self.upload_friend_audio if kind == "friend" else self.upload_group_audio
                sealed = (await uploader(uin, await element.fetch())).raw
            else:
                sealed = element.raw
            sender = self.send_friend_audio if kind == "friend" else self.send_group_audio
            return await sender(uin, sealed)
        if isinstance(element, MusicShare):
            raise TypeError("音乐分享无法因发送后无法获得消息元数据，无法使用 send_xxx_message API 发送，请直接调用底层 API")
        raise TypeError(f"无法发送元素: {element!r}")

    async def send_group_message(self, uin: int, chain: MessageChain) -> RawMessageReceipt:
        if isinstance(validated := self._validate_chain(chain), Element):
            return await self._send_special_element(uin, "group", validated)
        for idx, elem in enumerate(chain):
            chain.content[idx] = await self._validate_mm(uin, elem, self.upload_group_image)
        return await super().send_group_message(uin, _serialize_msg(chain))

    async def send_friend_message(self, uin: int, chain: MessageChain) -> RawMessageReceipt:
        if isinstance(validated := self._validate_chain(chain), Element):
            return await self._send_special_element(uin, "friend", validated)
        for idx, elem in enumerate(chain):
            chain.content[idx] = await self._validate_mm(uin, elem, self.upload_friend_image)
        return await super().send_friend_message(uin, _serialize_msg(chain))
