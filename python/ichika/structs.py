from enum import auto
from functools import total_ordering

from .utils import AutoEnum


class Gender(AutoEnum):
    """性别"""

    Male = auto()
    """男性"""
    Female = auto()
    """女性"""
    Unknown = auto()
    """未知"""


@total_ordering
class GroupPermission(AutoEnum):
    Owner = auto()
    """群主"""
    Admin = auto()
    """管理员"""
    Member = auto()
    """群成员"""

    def __lt__(self, other: object):
        if not isinstance(other, GroupPermission):
            return NotImplemented
        if self is GroupPermission.Owner:
            return False
        if self is GroupPermission.Admin:
            return other is GroupPermission.Owner
        return other is not GroupPermission.Member
