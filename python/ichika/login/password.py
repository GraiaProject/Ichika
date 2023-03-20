from __future__ import annotations

from enum import auto
from typing import Any, Callable, Literal, NoReturn, Optional, overload
from typing_extensions import Self

from loguru import logger as log

from ichika.utils import AsyncFn, AutoEnum, Decor


class PasswordLoginState(str, AutoEnum):
    Success = auto()
    AccountFrozen = auto()
    TooManySMSRequest = auto()
    DeviceLockLogin = auto()
    NeedCaptcha = auto()
    UnknownStatus = auto()
    DeviceLocked = auto()
    RequestSMS = auto()


class PasswordLoginCallbacks:
    def __init__(self, callbacks: dict[PasswordLoginState, Callable] | None = None):
        self.callbacks: dict[PasswordLoginState, Optional[Callable]] = {state: None for state in PasswordLoginState}
        self.callbacks.update(callbacks or {})

    @overload
    def set_handle(self, state: Literal[PasswordLoginState.DeviceLocked]) -> Decor[AsyncFn[[str, str], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[PasswordLoginState.RequestSMS]) -> Decor[AsyncFn[[str, str], str]]:
        ...

    @overload
    def set_handle(self, state: Literal[PasswordLoginState.NeedCaptcha]) -> Decor[AsyncFn[[str], str]]:
        ...

    @overload
    def set_handle(self, state: Literal[PasswordLoginState.Success]) -> Decor[AsyncFn[[], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[PasswordLoginState.DeviceLockLogin]) -> Decor[AsyncFn[[], Any]]:
        ...

    @overload
    def set_handle(
        self,
        state: Literal[PasswordLoginState.AccountFrozen],
    ) -> Decor[AsyncFn[[], NoReturn]]:
        ...

    @overload
    def set_handle(
        self,
        state: Literal[PasswordLoginState.TooManySMSRequest],
    ) -> Decor[AsyncFn[[], NoReturn]]:
        ...

    @overload
    def set_handle(
        self,
        state: Literal[PasswordLoginState.UnknownStatus],
    ) -> Decor[AsyncFn[[str, int], NoReturn]]:
        ...

    def set_handle(self, state) -> Decor[Callable]:
        def register_callback(func: Callable) -> Callable:
            self.callbacks[state] = func
            return func

        return register_callback

    def get_handle(self, state: str) -> Optional[Callable]:
        return self.callbacks.get(PasswordLoginState(state))

    @classmethod
    def default(cls) -> Self:
        cbs = cls({})
        S = PasswordLoginState

        @cbs.set_handle(S.NeedCaptcha)
        async def _(url: str):
            log.warning(f"请完成滑块验证，URL: {url}")
            return input("完成后请输入 ticket >").strip(" ")

        @cbs.set_handle(S.DeviceLocked)
        async def _(message: str, url: str):
            log.warning(message)
            log.warning(f"请完成设备锁验证，URL: {url}")
            input("请在完成后回车")

        @cbs.set_handle(S.RequestSMS)
        async def _(message: str, phone_number: str) -> str:
            log.warning(message)
            log.warning(f"已发送短信验证码至 {phone_number}")
            return input("请输入收到的短信验证码 >").strip(" ")

        @cbs.set_handle(S.AccountFrozen)
        async def _() -> NoReturn:
            msg = "无法登录：账号被冻结"
            raise RuntimeError(msg)

        @cbs.set_handle(S.TooManySMSRequest)
        async def _() -> NoReturn:
            msg = "短信请求次数过多，请稍后再试"
            raise RuntimeError(msg)

        @cbs.set_handle(S.UnknownStatus)
        async def _(message: str, code: int) -> NoReturn:
            msg = f"未知错误（代码 {code}）：{message}"
            raise RuntimeError(msg)

        @cbs.set_handle(S.Success)
        async def _() -> None:
            log.success("登录成功")

        @cbs.set_handle(S.DeviceLockLogin)
        async def _() -> None:
            log.info("尝试设备锁登录")

        return cbs
