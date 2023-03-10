from __future__ import annotations

from typing import Any

from graia.amnesia.message import MessageChain
from graia.amnesia.message.element import Element, Unknown

from .elements import TYPE_MAP
from .serializer import SERIALIZE_INV


def deserialize_message(elements: list[dict[str, Any]]) -> MessageChain:
    elem_seq: list[Element] = []
    for e_data in elements:
        cls = TYPE_MAP.get(e_data.pop("type"), None)
        if cls is None:
            print(e_data)
            elem_seq.append(Unknown("Unknown", e_data))
        else:
            elem_seq.append(cls(**e_data))
    return MessageChain(elem_seq)


def serialize_message(chain: MessageChain) -> list[dict[str, Any]]:
    res: list[dict[str, Any]] = []
    for elem in chain:
        if serializer := SERIALIZE_INV.get(elem.__class__):
            res.append(serializer(elem))
        else:
            raise TypeError(f"无法发送元素 {elem!r}")
    return res
