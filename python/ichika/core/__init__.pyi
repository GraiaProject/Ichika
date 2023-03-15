import datetime
from dataclasses import dataclass
from typing import Callable, Literal, TypedDict, TypeVar
from typing_extensions import Any, TypeAlias

from ..client import Client
from ..login import (
    BaseLoginCredentialStore,
    PasswordLoginCallbacks,
    Protocol,
    QRCodeLoginCallbacks,
)
from . import events as events

# region: build info

__Build_RustInfo = TypedDict(
    "_RustInfo",
    {
        "rustc": str,
        "rustc-version": str,
        "opt-level": str,
        "debug": bool,
        "jobs": int,
    },
)
__Build_HostInfo = TypedDict("__Build_HostInfo", {"triple": str})
__Build_TargetInfo = TypedDict(
    "__Build_TargetInfo",
    {
        "arch": str,
        "os": str,
        "family": str,
        "env": str,
        "triple": str,
        "endianness": str,
        "pointer-width": str,
        "profile": str,
    },
)
__BuildInfo = TypedDict(
    "_BuildInfo",
    {
        "build": __Build_RustInfo,
        "info-time": datetime.datetime,
        "dependencies": dict[str, str],
        "features": list[str],
        "host": __Build_HostInfo,
        "target": __Build_TargetInfo,
    },
)

__version__: str
__build__: __BuildInfo

# endregion: build info

async def password_login(
    uin: int,
    credential: str | bytes,
    use_sms: bool,
    protocol: Protocol,
    store: BaseLoginCredentialStore,
    event_callbacks: list[Callable],
    login_callbacks: PasswordLoginCallbacks,
) -> Client: ...
async def qrcode_login(
    uin: int,
    protocol: Protocol,
    store: BaseLoginCredentialStore,
    event_callbacks: list[Callable],
    login_callbacks: QRCodeLoginCallbacks,
) -> Client: ...

# region: client

_internal_repr = dataclass(frozen=True, init=False)

@_internal_repr
class AccountInfo:
    nickname: str
    age: int
    gender: int

@_internal_repr
class OtherClientInfo:
    app_id: int
    instance_id: int
    sub_platform: str
    device_kind: str

@_internal_repr
class Friend:
    uin: int
    nick: str
    remark: str
    face_id: int
    group_id: int

@_internal_repr
class FriendGroup:
    group_id: int
    name: str
    total_count: int
    online_count: int
    seq_id: int

@_internal_repr
class FriendList:
    total_count: int
    online_count: int
    def friends(self) -> tuple[Friend, ...]: ...
    def find_friend(self, uin: int) -> Friend | None: ...
    def friend_groups(self) -> tuple[FriendGroup, ...]: ...
    def find_friend_group(self, group_id: int) -> FriendGroup | None: ...

@_internal_repr
class Group:
    uin: int
    name: str
    memo: str
    owner_uin: int
    create_time: int
    level: int
    member_count: int
    max_member_count: int
    global_mute_timestamp: int
    mute_timestamp: int
    last_msg_seq: int

@_internal_repr
class Member:
    group_uin: int
    uin: int
    gender: int
    nickname: str
    card_name: str
    level: int
    join_time: int
    last_speak_time: int
    special_title: str
    special_title_expire_time: int
    mute_timestamp: int
    permission: int

_T = TypeVar("_T")

VTuple = tuple[_T, ...]

@_internal_repr
class RawMessageReceipt:
    seqs: VTuple[int]
    rands: VTuple[int]
    time: int
    kind: str
    target: int

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
    async def send_friend_message(self, uin: int, chain: list[dict[str, Any]]) -> RawMessageReceipt: ...
    async def upload_group_image(self, uin: int, data: bytes) -> dict[str, Any]: ...
    async def send_group_message(self, uin: int, chain: list[dict[str, Any]]) -> RawMessageReceipt: ...
    async def recall_friend_message(self, uin: int, time: int, seq: int, rand: int) -> None: ...
    async def recall_group_message(self, uin: int, seq: int, rand: int) -> None: ...
    async def modify_group_essence(self, uin: int, seq: int, rand: int, flag: bool) -> None: ...

# endregion: client

def face_id_from_name(name: str) -> int | None: ...
def face_name_from_id(id: int) -> str: ...
