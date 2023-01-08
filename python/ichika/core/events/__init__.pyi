from dataclasses import dataclass

from graia.amnesia.message import MessageChain

from . import structs as structs
from .structs import MemberInfo, MessageSource

internal_repr = dataclass(frozen=True, init=False)

@internal_repr
class LoginEvent:
    uin: int

@internal_repr
class GroupMessage:
    source: MessageSource
    content: MessageChain
    sender: MemberInfo
