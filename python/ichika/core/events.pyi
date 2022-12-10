from dataclasses import dataclass
from typing import Any

internal_repr = dataclass(frozen=True, init=False)

@internal_repr
class Login:
    uin: int

@internal_repr
class GroupMessage:
    sender: int
    group_uin: int
    group_name: int
    group_card: int
    def raw_elements(self) -> list[dict[str, Any]]: ...

@internal_repr
class GroupAudioMessage: ...

@internal_repr
class GroupMessageRecall: ...

@internal_repr
class GroupRequest: ...

@internal_repr
class SelfInvited: ...

@internal_repr
class NewMember: ...

@internal_repr
class GroupNameUpdate: ...

@internal_repr
class GroupMute: ...

@internal_repr
class GroupLeave: ...

@internal_repr
class GroupDisband: ...

@internal_repr
class MemberPermissionChange: ...

@internal_repr
class FriendMessage:
    target: int
    sender: int
    sender_name: str
    def raw_elements(self) -> list[dict[str, Any]]: ...

@internal_repr
class FriendAudioMessage: ...

@internal_repr
class FriendPoke: ...

@internal_repr
class FriendMessageRecall: ...

@internal_repr
class NewFriendRequest: ...

@internal_repr
class NewFriend: ...

@internal_repr
class DeleteFriend: ...

@internal_repr
class GroupTempMessage: ...

@internal_repr
class KickedOffline: ...

@internal_repr
class MSFOffline: ...
