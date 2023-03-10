"""基于 `ichika.core.PlumbingClient` 封装的高层 API"""

from graia.amnesia.message import MessageChain

from .core import PlumbingClient, RawMessageReceipt
from .message import serialize_message as _serialize_msg
from .message.elements import FlashImage, Image


class Client(PlumbingClient):
    async def upload_friend_image(self, uin: int, data: bytes) -> Image:
        image_dict = await super().upload_friend_image(uin, data)
        image_dict.pop("type")
        return Image(**image_dict)

    async def upload_group_image(self, uin: int, data: bytes) -> Image:
        image_dict = await super().upload_group_image(uin, data)
        image_dict.pop("type")
        return Image(**image_dict)

    async def send_group_message(self, uin: int, chain: MessageChain) -> RawMessageReceipt:
        for idx, elem in enumerate(chain):
            if Image._check(elem) and elem.raw is None:
                new_img = await self.upload_group_image(uin, await elem.fetch())
                if FlashImage._check(elem):
                    new_img = new_img.as_flash()
                chain.content[idx] = new_img
        return await super().send_group_message(uin, _serialize_msg(chain))

    async def send_friend_message(self, uin: int, chain: MessageChain) -> RawMessageReceipt:
        for idx, elem in enumerate(chain):
            if Image._check(elem) and elem.raw is None:
                new_img = await self.upload_friend_image(uin, await elem.fetch())
                if FlashImage._check(elem):
                    new_img = new_img.as_flash()
                chain.content[idx] = new_img
        return await super().send_friend_message(uin, _serialize_msg(chain))
