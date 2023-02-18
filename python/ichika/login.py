from enum import Enum
from typing import Any
from typing import Callable as _C
from typing import Literal as _L
from typing import TypeVar, overload

from ichika.client import Client


class Protocol(str, Enum):
    IPad = "IPad"
    AndroidPhone = "AndroidPhone"
    AndroidWatch = "AndroidWatch"
    MacOS = "MacOS"
    QiDian = "QiDian"


class LoginState(str, Enum):
    Success = "Success"
    AccountFrozen = "AccountFrozen"
    TooManySMSRequest = "TooManySMSRequest"
    DeviceLockLogin = "DeviceLockLogin"
    NeedCaptcha = "NeedCaptcha"
    UnknownStatus = "UnknownStatus"
    DeviceLocked = "DeviceLocked"
    RequestSMS = "RequestSMS"


_LoginState_T = TypeVar("_LoginState_T", bound=LoginState)
_CB = _C[[_LoginState_T], Any]


class LoginCallbacks:
    def __init__(self) -> None:
        self.callbacks = {}

    @overload
    def set_handle(self, state: _L[LoginState.Success], callback: _CB[_L[LoginState.Success]]) -> None:
        ...

    @overload
    def set_handle(self, state: _L[LoginState.AccountFrozen], callback: _CB[_L[LoginState.AccountFrozen]]) -> None:
        ...

    @overload
    def set_handle(
        self, state: _L[LoginState.TooManySMSRequest], callback: _CB[_L[LoginState.TooManySMSRequest]]
    ) -> None:
        ...

    @overload
    def set_handle(self, state: _L[LoginState.DeviceLockLogin], callback: _CB[_L[LoginState.DeviceLockLogin]]) -> None:
        ...

    @overload
    def set_handle(
        self, state: _L[LoginState.NeedCaptcha], callback: _C[[_L[LoginState.NeedCaptcha], str], str]
    ) -> None:
        ...

    @overload
    def set_handle(
        self,
        state: _L[LoginState.UnknownStatus],
        callback: _C[[_L[LoginState.UnknownStatus], int, str], Any],  # (state, status_code, message) -> Any
    ) -> None:
        ...

    @overload
    def set_handle(
        self,
        state: _L[LoginState.DeviceLocked],
        callback: _C[[_L[LoginState.DeviceLocked], str, str], Any],  # (state, verify_url, message) -> Any
    ) -> None:
        ...

    @overload
    def set_handle(
        self,
        state: _L[LoginState.RequestSMS],
        callback: _C[[_L[LoginState.RequestSMS], str, str], Any],  # (state, phone_number, message) -> Any
    ) -> None:
        ...

    def set_handle(self, state, callback):
        self.callbacks[state] = callback


def login_password(uin: int, password: str, protocol: Protocol, callbacks: LoginCallbacks) -> Client:
    ...


def login_qrcode(
    uin: int, protocol: _L[Protocol.AndroidWatch, Protocol.MacOS], show_qrcode: _C[[list[list[str]]], Any]
) -> Client:
    ...
