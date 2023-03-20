from enum import Enum
from typing import Awaitable, Callable, Generic, TypeVar
from typing_extensions import ParamSpec, TypeAlias

C_T = TypeVar("C_T", bound=Callable)
T = TypeVar("T")
R = TypeVar("R")
P = ParamSpec("P")
Decor: TypeAlias = Callable[[C_T], C_T]
AsyncFn: TypeAlias = Callable[P, Awaitable[T]]


class AutoEnum(Enum):
    _value_: str
    value: str

    def _generate_next_value_(name, *_):
        return name


class Ref(Generic[T]):
    def __init__(self, val: T) -> None:
        self.ref: T = val
