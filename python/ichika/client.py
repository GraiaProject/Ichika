"""基于 `ichika.core.PlumbingClient` 封装的高层 API"""
from __future__ import annotations

from graia.amnesia.message import Element, MessageChain

from .core import PlumbingClient, RawMessageReceipt
from .message import serialize_message as _serialize_msg
from .message.elements import (
    At,
    AtAll,
    Audio,
    Face,
    FlashImage,
    Image,
    SealedAudio,
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
        # Rich message types are: Reply, At, AtAll, Text, Face, Image / FlashImage
        if not chain:
            raise ValueError("无法发送空消息！")
        if any(not isinstance(elem, (At, AtAll, Text, Image, Face)) for elem in chain):
            if len(chain) > 1:
                raise ValueError("消息内混合了富文本和非富文本型消息！")
            return chain[0]
        return chain

    async def _send_special_element(self, uin: int, kind: str, element: Element) -> RawMessageReceipt:
        method = getattr(self, f"send_{kind}_{element.__class__.__name__.lower()}")
        if Audio._check(element):
            if element.raw is None:
                sealed = (await getattr(self, f"upload_{kind}_audio")(uin, await element.fetch())).raw
            else:
                sealed = element.raw
            return await method(uin, sealed)
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
