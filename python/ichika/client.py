"""基于 `ichika.core.PlumbingClient` 封装的高层 API"""
from __future__ import annotations

from typing import Any, Awaitable, Callable, Iterable, Literal, Protocol
from weakref import WeakValueDictionary

from graia.amnesia.message import Element, MessageChain

from .core import Friend, Group, PlumbingClient, RawMessageReceipt
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
    """HTTP 客户端协议"""

    def __call__(
        self, method: Literal["get", "post"], url: str, headers: dict[str, str], body: bytes
    ) -> Awaitable[bytes]:
        """发起 HTTP 请求

        :param method: 请求方法
        :param url: 请求地址
        :param headers: 请求头
        :param body: 请求体

        :return: 响应体
        """
        ...


def _uin(obj: Friend | Group | int) -> int:
    return obj if isinstance(obj, int) else obj.uin


def _chain_coerce(msg: str | Element | MessageChain | Iterable[str | Element]) -> MessageChain:
    if isinstance(msg, MessageChain):
        return msg
    if isinstance(msg, (str, Element)):
        msg = [msg]
    if isinstance(msg, Iterable):
        return MessageChain([Text(e) if isinstance(e, str) else e for e in msg])


class Client(PlumbingClient):
    """基于 [`PlumbingClient`][ichika.core.PlumbingClient] 封装的高层 API"""

    async def upload_friend_image(self, friend: int | Friend, data: bytes) -> Image:
        """上传好友图片

        :param friend: 好友 QQ 号或好友对象
        :param data: 图片数据

        :return: 图片元素
        """
        image_dict = await super().upload_friend_image(_uin(friend), data)
        image_dict.pop("type")
        return Image(**image_dict)

    async def upload_friend_audio(self, friend: int | Friend, data: bytes) -> Audio:
        """上传好友语音

        :param friend: 好友 QQ 号或好友对象
        :param data: 语音数据，应为 SILK/AMR 编码的音频数据

        :return: 语音元素
        """
        audio_dict = await super().upload_friend_audio(_uin(friend), data)
        audio_dict.pop("type")
        return Audio(**audio_dict)

    async def upload_group_image(self, group: int | Group, data: bytes) -> Image:
        """上传群图片

        :param group: 群号或群对象
        :param data: 图片数据

        :return: 图片元素
        """
        image_dict = await super().upload_group_image(_uin(group), data)
        image_dict.pop("type")
        return Image(**image_dict)

    async def upload_group_audio(self, group: int | Group, data: bytes) -> Audio:
        """上传群语音

        :param group: 群号或群对象
        :param data: 语音数据，应为 SILK/AMR 编码的音频数据

        :return: 语音元素
        """
        audio_dict = await super().upload_group_audio(_uin(group), data)
        audio_dict.pop("type")
        return Audio(**audio_dict)

    @classmethod
    def _parse_downloaded_fwd(cls, content: dict) -> ForwardMessage:
        if content.pop("type") == "Forward":
            content["content"] = [cls._parse_downloaded_fwd(sub) for sub in content.pop("content")]
        return ForwardMessage(**content)

    async def download_forward_msg(self, downloader: HttpClientProto, res_id: str) -> list[ForwardMessage]:
        """下载合并转发消息

        :param downloader: HTTP 客户端
        :param res_id: 资源 ID

        :return: 转发消息列表
        """
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

    async def upload_forward_msg(self, group: int | Group, msgs: list[ForwardMessage]) -> ForwardCard:
        """上传合并转发消息

        :param group: 用于标记的原始群号或群对象
        :param msgs: 转发消息列表

        :return: 转发卡片元素
        """
        res_id, file_name, content = await super().upload_forward_msg(
            _uin(group), [await self._prepare_forward(_uin(group), msg) for msg in msgs]
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
            sender = self.send_friend_music_share if kind == "friend" else self.send_group_music_share
            return await sender(uin, element)

        raise TypeError(f"无法发送元素: {element!r}")

    async def send_group_message(
        self, group: int | Group, chain: str | Element | MessageChain | Iterable[str | Element]
    ) -> RawMessageReceipt:
        """发送群消息

        :param group: 群号或群对象
        :param chain: 消息链

        :return: 消息发送凭据，可用于撤回
        """
        uin: int = _uin(group)
        chain = _chain_coerce(chain)
        if isinstance(validated := self._validate_chain(chain), Element):
            return await self._send_special_element(uin, "group", validated)
        for idx, elem in enumerate(chain):
            chain.content[idx] = await self._validate_mm(uin, elem, self.upload_group_image)
        return await super().send_group_message(uin, _serialize_msg(chain))

    async def send_friend_message(
        self, friend: int | Friend, chain: str | Element | MessageChain | Iterable[str | Element]
    ) -> RawMessageReceipt:
        """发送好友消息

        :param friend: 好友 QQ 号或好友对象
        :param chain: 消息链

        :return: 消息发送凭据，可用于撤回
        """
        uin: int = _uin(friend)
        chain = _chain_coerce(chain)
        if isinstance(validated := self._validate_chain(chain), Element):
            return await self._send_special_element(uin, "friend", validated)
        for idx, elem in enumerate(chain):
            chain.content[idx] = await self._validate_mm(uin, elem, self.upload_friend_image)
        return await super().send_friend_message(uin, _serialize_msg(chain))


CLIENT_REFS: WeakValueDictionary[int, Client] = WeakValueDictionary()
