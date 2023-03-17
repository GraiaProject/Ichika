from dataclasses import dataclass
from datetime import datetime

from graia.amnesia.message import MessageChain

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
class UnknownEvent: ...
