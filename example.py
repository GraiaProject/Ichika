from os import environ
from typing_extensions import Annotated

import creart
from graia.amnesia.message import MessageChain, Text
from graia.broadcast import Broadcast
from graiax.shortcut.text_parser import MatchTemplate, StartsWith
from launart import Launart

from ichika.client import Client
from ichika.core import Group
from ichika.graia import IchikaComponent
from ichika.graia.event import GroupMessage
from ichika.login import PathCredentialStore
from ichika.message.elements import Image

broadcast = creart.create(Broadcast)


@broadcast.receiver(GroupMessage)
async def listener(
    client: Client,
    group: Group,  # 获取事件发生的群组
    image: Annotated[MessageChain, StartsWith("来张图"), MatchTemplate([Text])]
    # 获取消息内容，其要求如下：
    # 1. 以“来张图”开头，后可跟至多一个空格
    # 2. 剩下的部分均为文字
):
    image_bytes = open(f"./images/{str(image)}.png", "rb").read()
    await client.send_group_message(group.uin, MessageChain([Text("图来了！\n"), Image.build(image_bytes)]))


mgr = Launart()
mgr.add_launchable(
    IchikaComponent(PathCredentialStore("./var/bots"), broadcast).add_password_login(
        int(environ["ACCOUNT"]), environ["PASSWORD"]
    )
)
mgr.launch_blocking(loop=broadcast.loop)
