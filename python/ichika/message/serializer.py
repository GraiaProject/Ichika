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
    Image,
    MarketFace,
)

SERIALIZE_INV: dict[type, Callable[[Any], dict[str, Any]]] = {}

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

        SERIALIZE_INV[elem_type] = wrapper
        return func

    return func_register


_serialize(Text)(lambda t: {"text": t.text})
_serialize(AtAll)(lambda _: {})
_serialize(At)(lambda t: {"target": t.target})
_serialize(Dice)(lambda t: {"value": t.value})
_serialize(FingerGuessing)(lambda t: {"choice": t.choice.name})
_serialize(Face)(lambda t: {"index": t.index})
_serialize(MarketFace)(lambda t: {"raw": t.raw})


@_serialize(Image)
def _(i: Image):
    if i.raw is None:
        raise ValueError
    return {"raw": i.raw}


@_serialize(FlashImage)
def _(i: FlashImage):
    if i.raw is None:
        raise ValueError
    return {"raw": i.raw}
