from __future__ import annotations

import functools
from typing import Any, Callable, TypeVar

from graia.amnesia.message import Element, Text

from ichika.utils import Decor

from .elements import (
    At,
    AtAll,
    Dice,
    Face,
    FingerGuessing,
    FlashImage,
    ForwardCard,
    Image,
    LightApp,
    MarketFace,
    Reply,
    RichMessage,
)

_SERIALIZE_INV: dict[type, Callable[[Any], dict[str, Any]]] = {}

Elem_T = TypeVar("Elem_T", bound=Element)


def _serialize(
    elem_type: type[Elem_T],
) -> Decor[Callable[[Elem_T], dict[str, Any]]]:
    def func_register(func: Callable[[Elem_T], dict[str, Any]]) -> Callable[[Elem_T], dict[str, Any]]:
        @functools.wraps(func)
        def wrapper(elem: Elem_T) -> dict[str, Any]:
            res = func(elem)
            res.setdefault("type", elem.__class__.__name__)
            return res

        _SERIALIZE_INV[elem_type] = wrapper
        return func

    return func_register


_serialize(Reply)(lambda t: {"seq": t.seq, "sender": t.sender, "time": int(t.time.timestamp()), "content": t.content})
_serialize(Text)(lambda t: {"text": t.text})
_serialize(AtAll)(lambda _: {})
_serialize(At)(lambda t: {"target": t.target, "display": t.display})
_serialize(Dice)(lambda t: {"value": t.value})
_serialize(FingerGuessing)(lambda t: {"choice": t.choice.name})
_serialize(Face)(lambda t: {"index": t.index})
_serialize(MarketFace)(lambda t: {"raw": t.raw})
_serialize(LightApp)(lambda t: {"content": t.content})
_serialize(RichMessage)(lambda t: {"service_id": t.service_id, "content": t.content})
_serialize(ForwardCard)(lambda t: {"service_id": 35, "content": t.content})


@_serialize(Image)
def _serialize_image(elem: Image):
    if elem.raw is None:
        raise ValueError
    return {"raw": elem.raw}


@_serialize(FlashImage)
def _serialize_flash_image(elem: FlashImage):
    if elem.raw is None:
        raise ValueError
    return {"raw": elem.raw}
