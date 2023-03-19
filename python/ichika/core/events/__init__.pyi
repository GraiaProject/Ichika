from dataclasses import dataclass
from datetime import datetime, timedelta
from typing import Literal

from graia.amnesia.message import MessageChain

from ichika.core import Group

from . import structs as structs
from .structs import FriendInfo, MemberInfo, MessageSource

internal_repr = dataclass(frozen=True, init=False)

@internal_repr
class LoginEvent:
    uin: int

@internal_repr
class GroupMessage:
    source: MessageSource
    content: MessageChain
    sender: MemberInfo

@internal_repr
class GroupRecallMessage:
    time: datetime
    author: MemberInfo
    operator: MemberInfo
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
    sender: MemberInfo

@internal_repr
class GroupNudge:
    sender: MemberInfo
    receiver: MemberInfo

@internal_repr
class FriendNudge:
    sender: FriendInfo

@internal_repr
class NewFriend:
    friend: FriendInfo

@internal_repr
class NewMember:
    member: MemberInfo

@internal_repr
class MemberLeaveGroup:
    group_uin: int
    member_uin: int

@internal_repr
class BotLeaveGroup:
    group_uin: int

@internal_repr
class GroupDisband:
    group_uin: int

@internal_repr
class FriendDeleted:
    friend_uin: int

@internal_repr
class GroupMute:
    group: Group
    operator: MemberInfo
    status: bool

@internal_repr
class MemberMute:
    operator: MemberInfo
    target: MemberInfo
    duration: timedelta | Literal[False]

@internal_repr
class BotMute:
    group: Group
    operator: MemberInfo
    duration: timedelta | Literal[False]

@internal_repr
class UnknownEvent: ...
