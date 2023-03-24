from datetime import datetime, timedelta
from typing import Literal, Optional, Type, TypedDict, Union
from typing_extensions import TypeGuard, TypeVar

from graia.amnesia.message import MessageChain

from ichika.core import Group, Member
from ichika.core.events.structs import FriendInfo, MessageSource


class LoginEvent(TypedDict):
    uin: int
    type_name: Literal["LoginEvent"]


class GroupMessage(TypedDict):
    source: MessageSource
    content: MessageChain
    group: Group
    sender: Member
    type_name: Literal["GroupMessage"]


class GroupRecallMessage(TypedDict):
    time: datetime
    group: Group
    author: Member
    operator: Member
    seq: int
    type_name: Literal["GroupRecallMessage"]


class FriendMessage(TypedDict):
    source: MessageSource
    content: MessageChain
    sender: FriendInfo
    type_name: Literal["FriendMessage"]


class FriendRecallMessage(TypedDict):
    time: datetime
    author: FriendInfo
    seq: int
    type_name: Literal["FriendRecallMessage"]


class TempMessage(TypedDict):
    source: MessageSource
    content: MessageChain
    group: Group
    sender: Member
    type_name: Literal["TempMessage"]


class GroupNudge(TypedDict):
    group: Group
    sender: Member
    receiver: Member
    type_name: Literal["GroupNudge"]


class FriendNudge(TypedDict):
    sender: FriendInfo
    type_name: Literal["FriendNudge"]


class NewFriend(TypedDict):
    friend: FriendInfo
    type_name: Literal["NewFriend"]


class NewMember(TypedDict):
    group: Group
    member: Member
    type_name: Literal["NewMember"]


class MemberLeaveGroup(TypedDict):
    group_uin: int
    member_uin: int
    type_name: Literal["MemberLeaveGroup"]


class GroupDisband(TypedDict):
    group_uin: int
    type_name: Literal["GroupDisband"]


class FriendDeleted(TypedDict):
    friend_uin: int
    type_name: Literal["FriendDeleted"]


class GroupMute(TypedDict):
    group: Group
    operator: Member
    status: bool
    type_name: Literal["GroupMute"]


class MemberMute(TypedDict):
    group: Group
    operator: Member
    target: Member
    duration: Union[timedelta, Literal[False]]
    type_name: Literal["MemberMute"]


class MemberPermissionChange(TypedDict):
    group: Group
    target: Member
    permission: int
    type_name: Literal["MemberPermissionChange"]


class _GroupInfo(TypedDict):
    name: str


class GroupInfoUpdate(TypedDict):
    group: Group
    operator: Member
    info: _GroupInfo
    type_name: Literal["GroupInfoUpdate"]


class NewFriendRequest(TypedDict):
    seq: int
    uin: int
    nickname: str
    message: str
    type_name: Literal["NewFriendRequest"]


class JoinGroupRequest(TypedDict):
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


class JoinGroupInvitation(TypedDict):
    seq: int
    time: datetime
    group_uin: int
    group_name: str
    invitor_uin: int
    invitor_nickname: str
    type_name: Literal["JoinGroupInvitation"]


class UnknownEvent(TypedDict):
    type_name: Literal["UnknownEvent"]
    internal_repr: str


Event = Union[
    LoginEvent,
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
    return e["type_name"] == type.__name__
