from types import ModuleType
from typing import Callable

from typing_extensions import Any

from .stubs import _LoginMethodTransfer

def init_log(m: ModuleType, /) -> None: ...

__version__: Any
__build__: Any

class Client(Any): ...

class Account:
    event_callbacks: list[Callable[[Any], Any]]
    def __init__(self, uin: int, data_folder: str, protocol: str) -> None: ...  # TODO: Literal
    async def login(self, method: _LoginMethodTransfer) -> Client: ...

def face_id_from_name(name: str) -> int | None: ...
def face_name_from_id(id: int) -> str: ...
