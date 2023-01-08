from dataclasses import dataclass

internal_repr = dataclass(frozen=True, init=False, eq=False)

@internal_repr
class MessageSource:
    seqs: tuple[int, ...]
    rands: tuple[int, ...]

@internal_repr
class GroupInfo:
    uin: int
    name: str

@internal_repr
class MemberInfo:
    uin: int
    name: str
    group: GroupInfo
