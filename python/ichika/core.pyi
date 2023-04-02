import asyncio
from dataclasses import dataclass
from datetime import datetime
from typing import Literal, TypeVar
from typing_extensions import Any, TypeAlias

from ichika.message.elements import MusicShare

from . import events as events
from .client import Client
from .login import (
    BaseLoginCredentialStore,
    PasswordLoginCallbacks,
    QRCodeLoginCallbacks,
)
from .message._sealed import SealedAudio as _SealedAudio

__version__: str
__build__: Any

_T_Event: TypeAlias = Any  # TODO

# Here, outside wrapper "login_XXX" ensures that a "task locals" can be acquired for event task execution.

async def password_login(
    uin: int,
    credential: str | bytes,
    use_sms: bool,
    protocol: str,
    store: BaseLoginCredentialStore,
    event_callbacks: list[asyncio.Queue[_T_Event]],
    login_callbacks: PasswordLoginCallbacks,
) -> Client:
    """使用密码登录。

    :param uin: 账号
    :param credential: 登录凭据，str 为密码，bytes 为 MD5 数据
    :param use_sms: 是否使用短信验证码验证设备锁
    :param protocol: 登录协议，为 AndroidPhone, AndroidPad, AndroidWatch, IPad, MacOS, QiDian 中的一个
    :param store: 登录凭据存储器
    :param event_callbacks: 事件队列
    :param login_callbacks: 用于解析登录的回调
    :return: 可操作的客户端
    """
    ...

async def qrcode_login(
    uin: int,
    protocol: str,
    store: BaseLoginCredentialStore,
    event_callbacks: list[asyncio.Queue[_T_Event]],
    login_callbacks: QRCodeLoginCallbacks,
) -> Client:
    """使用二维码登录。

    :param uin: 账号
    :param protocol: 登录协议，只能使用 AndroidWatch
    :param store: 登录凭据存储器
    :param event_callbacks: 事件队列
    :param login_callbacks: 用于解析登录的回调
    :return: 可操作的客户端
    """
    ...

# region: client

_internal_repr = dataclass(frozen=True, init=False)

@_internal_repr
class AccountInfo:
    """机器人账号信息"""

    nickname: str
    """机器人昵称"""
    age: int
    """机器人年龄"""
    gender: int  # TODO: note
    """机器人标注的性别"""

@_internal_repr
class OtherClientInfo:
    """获取到的其他客户端信息"""

    app_id: int
    """应用 ID"""
    instance_id: int
    """实例 ID"""
    sub_platform: str
    """子平台"""
    device_kind: str
    """设备类型"""

@_internal_repr
class Friend:
    """好友信息"""

    uin: int
    """账号 ID"""
    nick: str
    """好友昵称"""
    remark: str
    """好友备注"""
    face_id: int
    """未知"""
    group_id: int
    """好友分组 ID"""

@_internal_repr
class FriendGroup:
    """好友组"""

    group_id: int
    """分组 ID"""
    name: str
    """组名"""
    total_count: int
    """组内总好友数"""
    online_count: int
    """组内在线好友数"""
    seq_id: int
    """SEQ ID"""

@_internal_repr
class FriendList:
    """好友列表，你通过 API 获取到的顶层数据结构"""

    total_count: int
    """所有好友数"""
    online_count: int
    """在线好友数"""
    def friends(self) -> tuple[Friend, ...]:
        """获取好友列表。

        :return: 好友列表
        """
        ...
    def find_friend(self, uin: int) -> Friend | None:
        """查找好友。

        :param uin: 好友账号
        :return: 好友信息
        """
        ...
    def friend_groups(self) -> tuple[FriendGroup, ...]:
        """获取好友分组列表。

        :return: 好友分组列表
        """
        ...
    def find_friend_group(self, group_id: int) -> FriendGroup | None:
        """查找好友分组。

        :param group_id: 好友分组 ID
        :return: 好友分组信息
        """
        ...

@_internal_repr
class Group:
    """群组信息，请注意通过缓存获取的数据可能不精确"""

    uin: int
    """群号"""
    name: str
    """群名"""
    memo: str
    """群公告"""
    owner_uin: int
    """群主账号"""
    create_time: int  # TODO: datetime
    """群创建时间戳"""
    level: int
    """群等级"""
    member_count: int
    """群成员数量"""
    max_member_count: int
    """群最大成员数量"""
    global_mute_timestamp: int  # TODO: datetime
    """全局禁言时间戳"""
    mute_timestamp: int  # TODO: datetime
    """群禁言时间戳"""
    last_msg_seq: int
    """最后一条消息序列号"""

@_internal_repr
class Member:
    """群成员信息"""

    group_uin: int
    """群号"""
    uin: int
    """账号"""
    gender: int
    """性别"""
    nickname: str
    """昵称"""
    card_name: str
    """群名片"""
    level: int
    """成员等级"""
    join_time: int
    """加入时间"""
    last_speak_time: int
    """最后发言时间"""
    special_title: str
    """特殊头衔"""
    special_title_expire_time: int
    """特殊头衔过期时间"""
    mute_timestamp: int
    """禁言时间戳"""
    permission: int  # TODO: Enum
    """权限

    - 0: 群员
    - 1: 管理员
    - 2: 群主
    """

_T = TypeVar("_T")

VTuple = tuple[_T, ...]

@_internal_repr
class RawMessageReceipt:
    seqs: VTuple[int]
    """消息 SEQ ID"""
    rands: VTuple[int]
    """消息随机数"""
    time: int
    """发送时间戳"""
    kind: str
    """消息类型，为 `group` 与 `friend` 中一个"""
    target: int
    """发送目标"""

@_internal_repr
class OCRText:
    detected_text: str
    confidence: int
    polygon: VTuple[tuple[int, int]] | None
    advanced_info: str

@_internal_repr
class OCRResult:
    texts: VTuple[OCRText]
    language: str

__OnlineStatus: TypeAlias = (  # TODO: Wrapper
    tuple[int, str]  # (face_index, wording)
    | tuple[
        Literal[False],
        Literal[
            11,  # 在线
            21,  # 离线，效果未知
            31,  # 离开
            41,  # 隐身
            50,  # 忙
            60,  # Q 我吧
            70,  # 请勿打扰
        ],
    ]
    | tuple[
        Literal[True],
        Literal[
            1000,  # 当前电量
            1028,  # 听歌中
            1040,  # 星座运势
            1030,  # 今日天气
            1069,  # 遇见春天
            1027,  # Timi中
            1064,  # 吃鸡中
            1051,  # 恋爱中
            1053,  # 汪汪汪
            1019,  # 干饭中
            1018,  # 学习中
            1032,  # 熬夜中
            1050,  # 打球中
            1011,  # 信号弱
            1024,  # 在线学习
            1017,  # 游戏中
            1022,  # 度假中
            1021,  # 追剧中
            1020,  # 健身中
        ],
    ]
)

class PlumbingClient:
    # [impl 1]
    @property
    def uin(self) -> int: ...
    @property
    def online(self) -> bool: ...
    async def keep_alive(self) -> None: ...
    async def stop(self) -> None: ...
    async def get_account_info(self) -> AccountInfo: ...
    async def set_account_info(
        self,
        *,
        name: str | None = None,
        email: str | None = None,
        personal_note: str | None = None,
        company: str | None = None,
        college: str | None = None,
        signature: str = ...,
    ) -> None: ...
    async def get_other_clients(self) -> VTuple[OtherClientInfo]: ...
    async def modify_online_status(self, status: __OnlineStatus) -> None: ...
    async def image_ocr(self, url: str, md5: str, width: int, height: int) -> OCRResult: ...
    # [impl 2]
    async def get_friend_list(self) -> FriendList: ...
    async def get_friend_list_raw(self) -> FriendList: ...
    async def get_friends(self) -> VTuple[Friend]: ...
    async def find_friend(self, uin: int) -> Friend | None: ...
    async def nudge_friend(self, uin: int) -> None: ...
    async def delete_friend(self, uin: int) -> None: ...
    # [impl 3]
    async def get_group(self, uin: int) -> Group: ...
    async def get_group_raw(self, uin: int) -> Group: ...
    async def find_group(self, uin: int) -> Group | None: ...
    async def get_groups(self) -> VTuple[Group]: ...
    async def get_group_admins(self, uin: int) -> list[tuple[int, int]]: ...
    async def mute_group(self, uin: int, mute: bool) -> None: ...
    async def quit_group(self, uin: int) -> None: ...
    async def modify_group_info(self, uin: int, *, memo: str | None = None, name: str | None = None) -> None: ...
    async def group_sign_in(self, uin: int) -> None: ...
    # [impl 4]
    async def get_member(self, group_uin: int, uin: int) -> Member: ...
    async def get_member_raw(self, group_uin: int, uin: int) -> Member: ...
    async def nudge_member(self, group_uin: int, uin: int) -> None: ...
    # Duration -> 0: Unmute
    async def mute_member(self, group_uin: int, uin: int, duration: int) -> None: ...
    async def kick_member(self, group_uin: int, uin: int, msg: str, block: bool) -> None: ...
    async def modify_member_special_title(self, group_uin: int, uin: int, special_title: str) -> None: ...
    async def modify_member_card(self, group_uin: int, uin: int, card_name: str) -> None: ...
    async def modify_member_admin(self, group_uin: int, uin: int, admin: bool) -> None: ...
    # [impl 5]
    async def upload_friend_image(self, uin: int, data: bytes) -> dict[str, Any]: ...
    async def upload_friend_audio(self, uin: int, data: bytes) -> dict[str, Any]: ...
    async def upload_group_image(self, uin: int, data: bytes) -> dict[str, Any]: ...
    async def upload_group_audio(self, uin: int, data: bytes) -> dict[str, Any]: ...
    async def send_friend_audio(self, uin: int, audio: _SealedAudio) -> RawMessageReceipt: ...
    async def send_group_audio(self, uin: int, audio: _SealedAudio) -> RawMessageReceipt: ...
    async def send_friend_music_share(self, uin: int, share: MusicShare) -> None: ...
    async def send_group_music_share(self, uin: int, share: MusicShare) -> None: ...
    # [impl 6]
    async def send_friend_message(self, uin: int, chain: list[dict[str, Any]]) -> RawMessageReceipt: ...
    async def send_group_message(self, uin: int, chain: list[dict[str, Any]]) -> RawMessageReceipt: ...
    async def recall_friend_message(self, uin: int, time: int, seq: int, rand: int) -> None: ...
    async def recall_group_message(self, uin: int, seq: int, rand: int) -> None: ...
    async def modify_group_essence(self, uin: int, seq: int, rand: int, flag: bool) -> None: ...
    # [impl 7]
    async def process_join_group_request(
        self, seq: int, request_uin: int, group_uin: int, accept: bool, block: bool, message: str
    ) -> None: ...
    async def process_group_invitation(self, seq: int, invitor_uin: int, group_uin: int, accept: bool) -> None: ...
    async def process_new_friend_request(self, seq: int, request_uin: int, accept: bool) -> None: ...

# endregion: client

def face_id_from_name(name: str) -> int | None: ...
def face_name_from_id(id: int) -> str: ...
@_internal_repr
class MessageSource:
    """消息元信息"""

    seqs: tuple[int, ...]
    """消息的 SEQ
    建议搭配聊天类型与上下文 ID （例如 `("group", 123456, seq)`）作为索引的键
    """
    rands: tuple[int, ...]
    """消息的随机信息，撤回需要"""
    time: datetime
    """消息发送时间"""

@_internal_repr
class FriendInfo:
    """事件中的好友信息"""

    uin: int
    """好友账号"""
    nickname: str
    """好友实际昵称"""
