"""基于 `ichika.core.PlumbingClient` 封装的高层 API"""
from __future__ import annotations

from graia.amnesia.message import Element, MessageChain

from .core import PlumbingClient, RawMessageReceipt
from .message import _serialize_message as _serialize_msg
from .message.elements import (
    At,
    AtAll,
    Audio,
    Face,
    FlashImage,
    Image,
    MusicShare,
    Reply,
    Text,
)


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

    async def _validate_chain(self, chain: MessageChain) -> MessageChain | Element:
        if not chain:
            raise ValueError("无法发送空消息！")
        if any(not isinstance(elem, (Reply, At, AtAll, Text, Image, Face)) for elem in chain):
            if len(chain) > 1:
                raise ValueError("消息内混合了富文本和非富文本型消息！")
            return chain[0]
        return chain

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
        if isinstance(validated := await self._validate_chain(chain), Element):
            return await self._send_special_element(uin, "group", validated)
        for idx, elem in enumerate(chain):
            if Image._check(elem) and elem.raw is None:
                new_img = await self.upload_group_image(uin, await elem.fetch())
                if FlashImage._check(elem):
                    new_img = new_img.as_flash()
                chain.content[idx] = new_img
        return await super().send_group_message(uin, _serialize_msg(chain))

    async def send_friend_message(self, uin: int, chain: MessageChain) -> RawMessageReceipt:
        if isinstance(validated := await self._validate_chain(chain), Element):
            return await self._send_special_element(uin, "friend", validated)
        for idx, elem in enumerate(chain):
            if Image._check(elem) and elem.raw is None:
                new_img = await self.upload_friend_image(uin, await elem.fetch())
                if FlashImage._check(elem):
                    new_img = new_img.as_flash()
                chain.content[idx] = new_img
        return await super().send_friend_message(uin, _serialize_msg(chain))
