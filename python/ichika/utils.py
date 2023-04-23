from __future__ import annotations

import contextlib
import sys
from enum import Enum
from typing import Any, Awaitable, Callable, Generic, Literal, TypeVar, Union
from typing_extensions import Annotated, ParamSpec, TypeAlias, get_args
from typing_extensions import get_origin as typing_get_origin

C_T = TypeVar("C_T", bound=Callable)
T = TypeVar("T")
R = TypeVar("R")
P = ParamSpec("P")
Decor: TypeAlias = Callable[[C_T], C_T]
AsyncFn: TypeAlias = Callable[P, Awaitable[T]]


class AutoEnum(Enum):
    """以名字为值的自动枚举"""

    _value_: str
    value: str

    def _generate_next_value_(name, *_):
        return name


class Ref(Generic[T]):
    def __init__(self, val: T) -> None:
        self.ref: T = val


AnnotatedType: type = type(Annotated[int, lambda x: x > 0])
if sys.version_info >= (3, 10):
    import types

    Unions = (Union, types.UnionType)
else:
    Unions = (Union,)


def get_origin(obj: Any) -> Any:
    return typing_get_origin(obj) or obj


def generic_issubclass(cls: type, par: Union[type, Any, tuple[type, ...]]) -> bool:
    if par is Any:
        return True
    if cls is type(None) and par is None:
        return True
    with contextlib.suppress(TypeError):
        if isinstance(par, AnnotatedType):
            return generic_issubclass(cls, get_args(par)[0])
        if isinstance(par, type):
            return issubclass(cls, par)
        if get_origin(par) in Unions:
            return any(generic_issubclass(cls, p) for p in get_args(par))
        if isinstance(par, TypeVar):
            if par.__constraints__:
                return any(generic_issubclass(cls, p) for p in par.__constraints__)
            if par.__bound__:
                return generic_issubclass(cls, par.__bound__)
        if isinstance(par, tuple):
            return any(generic_issubclass(cls, p) for p in par)
        if issubclass(cls, get_origin(par)):
            return True
    return False
