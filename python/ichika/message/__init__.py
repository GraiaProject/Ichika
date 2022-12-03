from graia.amnesia.message import MessageChain
from graia.amnesia.message.element import Element, Unknown
from typing import Any
from .elements import TYPE_MAP


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
