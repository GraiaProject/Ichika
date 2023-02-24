from __future__ import annotations

from typing import Protocol, runtime_checkable


@runtime_checkable
class QRCodeRenderer(Protocol):
    def render(self, data: list[list[bool]], /) -> str:
        ...


from .dense1x2 import Dense1x2 as Dense1x2
