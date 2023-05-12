import contextlib
from dataclasses import dataclass
from datetime import datetime, timedelta
from typing import Dict, Literal, Optional, Type, TypedDict, TypeVar, Union
from typing_extensions import get_type_hints

from graia.amnesia.message import MessageChain
from graia.broadcast.entities.event import BaseDispatcher, Dispatchable
from graia.broadcast.entities.signatures import Force
from graia.broadcast.interfaces.dispatcher import DispatcherInterface as DI

from ichika.core import Friend, Group, Member, MessageSource
from ichika.utils import generic_issubclass


class NoneDispatcher(BaseDispatcher):
    """给 Optional[...] 提供 None 的 Dispatcher"""

    @staticmethod
    async def catch(interface: DI):
        if NoneDispatcher in interface.current_oplog:  # FIXME: Workaround
            return None
            # oplog cached NoneDispatcher, which is undesirable
            # return "None" causes it to clear the cache
            # Then all the dispatchers are revisited
            # So that "None" is normally dispatched.
        if generic_issubclass(type(None), interface.annotation):
            return Force(None)


class SourceDispatcher(BaseDispatcher):
    @staticmethod
    async def catch(interface: DI):
        event = interface.event
        if isinstance(event, MessageEvent) and generic_issubclass(MessageSource, interface.annotation):
            return event.source


class MessageChainDispatcher(BaseDispatcher):
    @staticmethod
    async def catch(interface: DI):
        event = interface.event
        if isinstance(event, MessageEvent) and generic_issubclass(MessageChain, interface.annotation):
            return event.content


class SenderDispatcher(BaseDispatcher):
    @staticmethod
    async def catch(interface: DI):
        event = interface.event
        with contextlib.suppress(TypeError):
            if isinstance(event, MessageEvent) and generic_issubclass(event.sender.__class__, interface.annotation):
                return event.sender


class GroupDispatcher(BaseDispatcher):
    @staticmethod
    async def catch(interface: DI):
        event = interface.event
        if generic_issubclass(Group, interface.annotation):
            return event.group


_DISPATCHER_MAP: Dict[type, Type[BaseDispatcher]] = {
    MessageSource: SourceDispatcher,
    MessageChain: MessageChainDispatcher,
    Group: GroupDispatcher,
}

_Dispatch_T = TypeVar("_Dispatch_T", bound=Dispatchable)


def auto_dispatch(event_cls: Type[_Dispatch_T]) -> Type[_Dispatch_T]:
    mixins: set[Type[BaseDispatcher]] = {NoneDispatcher}
    type_map: dict[type, set[str]] = {}

    for name, typ in get_type_hints(event_cls).items():
        if name == "sender":
            mixins.add(SenderDispatcher)
        elif dispatcher := _DISPATCHER_MAP.get(typ):
            mixins.add(dispatcher)
        else:
            type_map.setdefault(typ, set()).add(name)

    class Dispatcher(BaseDispatcher):
        mixin = list(mixins)
        _type_dispatch: Dict[type, str] = {t: next(iter(ns)) for t, ns in type_map.items() if len(ns) <= 1}
        _name_dispatch: Dict[str, type] = {n: t for t, ns in type_map.items() if len(ns) > 1 for n in ns}

        @classmethod
        async def catch(cls, interface: DI):
            anno, name, event = interface.annotation, interface.name, interface.event
            if name in cls._name_dispatch and generic_issubclass(cls._name_dispatch[name], anno):
                return getattr(event, name)
            if generic_issubclass(event_cls, anno):
                return event
            for t, target_name in cls._type_dispatch.items():
                if generic_issubclass(t, anno):
                    return getattr(event, target_name)

    Dispatcher.__module__ = event_cls.__module__
    Dispatcher.__qualname__ = f"{event_cls.__qualname__}.Dispatcher"

    event_cls.Dispatcher = Dispatcher
    return event_cls


class MessageEvent(Dispatchable):
    source: MessageSource
    content: MessageChain
    sender: Union[Member, Friend]


class GroupEvent(Dispatchable):
    group: Group


@dataclass
@auto_dispatch
class GroupMessage(MessageEvent, GroupEvent):
    source: MessageSource
    content: MessageChain
    group: Group
    sender: Member


@dataclass
@auto_dispatch
class FriendMessage(MessageEvent):
    source: MessageSource
    content: MessageChain
    sender: Friend


@dataclass
@auto_dispatch
class TempMessage(MessageEvent):
    source: MessageSource
    content: MessageChain
    group: Group
    sender: Member


@dataclass
@auto_dispatch
class GroupRecallMessage(Dispatchable):
    time: datetime
    group: Group
    author: Member
    operator: Member
    seq: int


@dataclass
@auto_dispatch
class FriendRecallMessage(Dispatchable):
    time: datetime
    author: Friend
    seq: int


@dataclass
@auto_dispatch
class GroupNudge(Dispatchable):
    group: Group
    sender: Member
    receiver: Member


@dataclass
@auto_dispatch
class FriendNudge(Dispatchable):
    sender: Friend


@dataclass
@auto_dispatch
class NewFriend(Dispatchable):
    friend: Friend


@dataclass
@auto_dispatch
class NewMember(Dispatchable):
    group: Group
    member: Member


@dataclass
@auto_dispatch
class MemberLeaveGroup(Dispatchable):
    group_uin: int
    member_uin: int


@dataclass
@auto_dispatch
class GroupDisband(Dispatchable):
    group_uin: int


@dataclass
@auto_dispatch
class FriendDeleted(Dispatchable):
    friend_uin: int


@dataclass
@auto_dispatch
class GroupMute(Dispatchable):
    group: Group
    operator: Member
    status: bool


@dataclass
@auto_dispatch
class MemberMute(Dispatchable):
    group: Group
    operator: Member
    target: Member
    duration: Union[timedelta, Literal[False]]


@dataclass
@auto_dispatch
class MemberPermissionChange(Dispatchable):
    group: Group
    target: Member
    permission: int


class _GroupInfo(TypedDict):
    name: str


@dataclass
@auto_dispatch
class GroupInfoUpdate(Dispatchable):
    group: Group
    operator: Member
    info: _GroupInfo


@dataclass
@auto_dispatch
class NewFriendRequest(Dispatchable):
    seq: int
    uin: int
    nickname: str
    message: str


@dataclass
@auto_dispatch
class JoinGroupRequest(Dispatchable):
    seq: int
    time: datetime
    group_uin: int
    group_name: str
    request_uin: int
    request_nickname: str
    suspicious: bool
    invitor_uin: Optional[int]
    invitor_nickname: Optional[str]


@dataclass
@auto_dispatch
class JoinGroupInvitation(Dispatchable):
    seq: int
    time: datetime
    group_uin: int
    group_name: str
    invitor_uin: int
    invitor_nickname: str


@dataclass
@auto_dispatch
class UnknownEvent(Dispatchable):
    internal_repr: str


EVENT_TYPES = {
    cls.__name__: cls
    for cls in (
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
    )
}
