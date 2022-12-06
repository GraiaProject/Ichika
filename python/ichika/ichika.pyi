import datetime
from dataclasses import dataclass
from types import ModuleType
from typing import Callable, TypedDict

from typing_extensions import Any

from .stubs import _LoginMethodTransfer

__Build_RustInfo = TypedDict(
    "_RustInfo",
    {
        "rustc": str,
        "rustc-version": str,
        "opt-level": str,
        "debug": bool,
        "jobs": int,
    },
)
__Build_HostInfo = TypedDict("__Build_HostInfo", {"triple": str})
__Build_TargetInfo = TypedDict(
    "__Build_TargetInfo",
    {
        "arch": str,
        "os": str,
        "family": str,
        "env": str,
        "triple": str,
        "endianness": str,
        "pointer-width": str,
        "profile": str,
    },
)
__BuildInfo = TypedDict(
    "_BuildInfo",
    {
        "build": __Build_RustInfo,
        "info-time": datetime.datetime,
        "dependencies": dict[str, str],
        "features": list[str],
        "host": __Build_HostInfo,
        "target": __Build_TargetInfo,
    },
)

def init_log(m: ModuleType, /) -> None: ...

__version__: str
__build__: __BuildInfo

class Client(Any): ...

class Account:
    event_callbacks: list[Callable[[Any], Any]]
    def __init__(self, uin: int, data_folder: str, protocol: str) -> None: ...  # TODO: Literal
    async def login(self, method: _LoginMethodTransfer) -> Client: ...

class PlumbingClient:
    async def keep_alive(self) -> None: ...
    @property
    def online(self) -> bool: ...
    @dataclass(frozen=True)
    class __AccountInfo:
        nickname: str
        age: int
        gender: int
    async def account_info(self) -> __AccountInfo: ...
    @dataclass(frozen=True)
    class __OtherClientInfo:
        app_id: int
        instance_id: int
        sub_platform: str
        device_kind: str
    async def other_clients(self) -> list[__OtherClientInfo]: ...

def face_id_from_name(name: str) -> int | None: ...
def face_name_from_id(id: int) -> str: ...
