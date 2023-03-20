from dataclasses import dataclass
from datetime import datetime, timedelta
from typing import Literal, TypedDict

from graia.amnesia.message import MessageChain

from ichika.core import Group, Member

from . import structs as structs
from .structs import FriendInfo, MessageSource

internal_repr = dataclass(frozen=True, init=False)

@internal_repr
class LoginEvent:
    uin: int

@internal_repr
class GroupMessage:
    source: MessageSource
    content: MessageChain
    group: Group
    sender: Member

@internal_repr
class GroupRecallMessage:
    time: datetime
    group: Group
    author: Member
    operator: Member
    seq: int

@internal_repr
class FriendMessage:
    source: MessageSource
    content: MessageChain
    sender: FriendInfo

@internal_repr
class FriendRecallMessage:
    time: datetime
    author: FriendInfo
    seq: int

@internal_repr
class TempMessage:
    source: MessageSource
    content: MessageChain
    group: Group
    sender: Member

@internal_repr
class GroupNudge:
    group: Group
    sender: Member
    receiver: Member

@internal_repr
class FriendNudge:
    sender: FriendInfo

@internal_repr
class NewFriend:
    friend: FriendInfo

@internal_repr
class NewMember:
    group: Group
    member: Member

@internal_repr
class MemberLeaveGroup:
    group_uin: int
    member_uin: int

@internal_repr
class GroupDisband:
    group_uin: int

@internal_repr
class FriendDeleted:
    friend_uin: int

@internal_repr
class GroupMute:
    group: Group
    operator: Member
    status: bool

@internal_repr
class MemberMute:
    group: Group
    operator: Member
    target: Member
    duration: timedelta | Literal[False]

@internal_repr
class MemberPermissionChange:
    group: Group
    target: Member
    permission: int

class __GroupInfo(TypedDict):
    name: str

@internal_repr
class GroupInfoUpdate:
    group: Group
    operator: Member
    info: __GroupInfo

@internal_repr
class UnknownEvent: ...
