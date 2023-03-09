from dataclasses import dataclass
from datetime import datetime

from ichika.core import Group

internal_repr = dataclass(frozen=True, init=False, eq=False)

@internal_repr
class MessageSource:
    seqs: tuple[int, ...]
    rands: tuple[int, ...]
    time: datetime

@internal_repr
class MemberInfo:
    uin: int
    name: str
    group: Group
