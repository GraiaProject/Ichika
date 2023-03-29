from dataclasses import dataclass
from datetime import datetime

_internal_repr = dataclass(frozen=True, init=False, eq=False)

@_internal_repr
class MessageSource:
    """消息元信息"""

    seqs: tuple[int, ...]
    """消息的 SEQ

    建议搭配聊天类型与上下文 ID （例如 `("group", 123456, seq)`）作为索引的键
    """
    rands: tuple[int, ...]
    """消息的随机信息，撤回需要"""
    time: datetime
    """消息发送时间"""

@_internal_repr
class FriendInfo:
    """事件中的好友信息"""

    uin: int
    """好友账号"""
    nickname: str
    """好友实际昵称"""
