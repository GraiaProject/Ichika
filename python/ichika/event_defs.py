"""基于 [`TypedDict`][typing.TypedDict] 的事件定义。

对接本框架的下游开发者应该参考此处。
"""
from datetime import datetime, timedelta
from typing import Literal, Optional, Type, TypedDict, Union
from typing_extensions import TypeGuard, TypeVar

from graia.amnesia.message import MessageChain

from ichika.client import Client
from ichika.core import Friend, Group, Member, MessageSource


class BaseEvent(TypedDict):
    client: Client
    """事件所属的 [`Client`][ichika.client.Client] 对象"""


class GroupMessage(BaseEvent):
    source: MessageSource
    content: MessageChain
    group: Group
    sender: Member
    type_name: Literal["GroupMessage"]


class GroupRecallMessage(BaseEvent):
    time: datetime
    group: Group
    author: Member
    operator: Member
    seq: int
    type_name: Literal["GroupRecallMessage"]


class FriendMessage(BaseEvent):
    source: MessageSource
    content: MessageChain
    sender: Friend
    type_name: Literal["FriendMessage"]


class FriendRecallMessage(BaseEvent):
    time: datetime
    author: Friend
    seq: int
    type_name: Literal["FriendRecallMessage"]


class TempMessage(BaseEvent):
    source: MessageSource
    content: MessageChain
    group: Group
    sender: Member
    type_name: Literal["TempMessage"]


class GroupNudge(BaseEvent):
    group: Group
    sender: Member
    receiver: Member
    type_name: Literal["GroupNudge"]


class FriendNudge(BaseEvent):
    sender: Friend
    type_name: Literal["FriendNudge"]


class NewFriend(BaseEvent):
    friend: Friend
    type_name: Literal["NewFriend"]


class NewMember(BaseEvent):
    group: Group
    member: Member
    type_name: Literal["NewMember"]


class MemberLeaveGroup(BaseEvent):
    group_uin: int
    member_uin: int
    type_name: Literal["MemberLeaveGroup"]


class GroupDisband(BaseEvent):
    group_uin: int
    type_name: Literal["GroupDisband"]


class FriendDeleted(BaseEvent):
    friend_uin: int
    type_name: Literal["FriendDeleted"]


class GroupMute(BaseEvent):
    group: Group
    operator: Member
    status: bool
    type_name: Literal["GroupMute"]


class MemberMute(BaseEvent):
    group: Group
    operator: Member
    target: Member
    duration: Union[timedelta, Literal[False]]
    type_name: Literal["MemberMute"]


class MemberPermissionChange(BaseEvent):
    group: Group
    target: Member
    permission: int
    type_name: Literal["MemberPermissionChange"]


class _GroupInfo(BaseEvent):
    name: str


class GroupInfoUpdate(BaseEvent):
    group: Group
    operator: Member
    info: _GroupInfo
    type_name: Literal["GroupInfoUpdate"]


class NewFriendRequest(BaseEvent):
    seq: int
    uin: int
    nickname: str
    message: str
    type_name: Literal["NewFriendRequest"]


class JoinGroupRequest(BaseEvent):
    seq: int
    time: datetime
    group_uin: int
    group_name: str
    request_uin: int
    request_nickname: str
    suspicious: bool
    invitor_uin: Optional[int]
    invitor_nickname: Optional[str]
    type_name: Literal["JoinGroupRequest"]


class JoinGroupInvitation(BaseEvent):
    seq: int
    time: datetime
    group_uin: int
    group_name: str
    invitor_uin: int
    invitor_nickname: str
    type_name: Literal["JoinGroupInvitation"]


class UnknownEvent(BaseEvent):
    """未知事件"""

    type_name: Literal["UnknownEvent"]
    internal_repr: str
    """事件的内部表示"""


Event = Union[
    GroupMessage,
    GroupRecallMessage,
    FriendMessage,
    FriendRecallMessage,
    TempMessage,
    GroupNudge,
    FriendNudge,
    NewFriend,
    NewMember,
    MemberLeaveGroup,
    GroupDisband,
    FriendDeleted,
    GroupMute,
    MemberMute,
    MemberPermissionChange,
    GroupInfoUpdate,
    NewFriendRequest,
    JoinGroupRequest,
    JoinGroupInvitation,
    UnknownEvent,
]

_T_Event = TypeVar("_T_Event", bound=Event)


def check_event(e: Event, type: Type[_T_Event]) -> TypeGuard[_T_Event]:
    """检查事件是否为指定类型。

    :param e: 事件对象
    :param type: 事件类型

    :return: 事件是否为指定类型
    """
    return e["type_name"] == type.__name__
