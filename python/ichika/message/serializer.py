import functools
from typing import Any, Callable, TypeVar

from graia.amnesia.message import Element, Text

from .elements import At, AtAll, Dice, Face, FingerGuessing, MarketFace

_serialize_inventory = {}

Elem_T = TypeVar("Elem_T", bound=Element)


def _serialize(
    elem_type: type[Elem_T],
) -> Callable[[Callable[[Elem_T], dict[str, Any]]], Callable[[Elem_T], dict[str, Any]]]:
    def func_register(func: Callable[[Elem_T], dict[str, Any]]) -> Callable[[Elem_T], dict[str, Any]]:
        @functools.wraps(func)
        def wrapper(elem: Elem_T) -> dict[str, Any]:
            res = func(elem)
            res.setdefault("type", elem.__class__.__name__)
            return res

        _serialize_inventory[elem_type] = wrapper
        return func

    return func_register


_serialize(Text)(lambda t: {"text": t.text})
_serialize(AtAll)(lambda _: {})
_serialize(At)(lambda t: {"target": t.target})
_serialize(Dice)(lambda t: {"value": t.value})
_serialize(FingerGuessing)(lambda t: {"choice": t.choice.name})
_serialize(Face)(lambda t: {"index": t.index})
_serialize(MarketFace)(lambda t: {"raw": t.raw})
