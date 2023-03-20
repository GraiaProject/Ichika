from __future__ import annotations

from enum import auto
from typing import Any, Callable, Literal, Optional, overload
from typing_extensions import Self

from loguru import logger as log

from ichika.utils import AsyncFn, AutoEnum, Decor, P, Ref

from .render import Dense1x2 as Dense1x2
from .render import QRCodeRenderer as QRCodeRenderer


class QRCodeLoginState(str, AutoEnum):
    WaitingForScan = auto()
    WaitingForConfirm = auto()
    Canceled = auto()
    Timeout = auto()
    Success = auto()
    DisplayQRCode = auto()
    UINMismatch = auto()


class QRCodeLoginCallbacks:
    def __init__(self, callbacks: dict[QRCodeLoginState, Callable] | None = None, interval: float = 5.0):
        self.callbacks: dict[QRCodeLoginState, Optional[Callable]] = {state: None for state in QRCodeLoginState}
        self.callbacks.update(callbacks or {})
        self.interval: float = interval

    @overload
    def set_handle(self, state: Literal[QRCodeLoginState.WaitingForScan]) -> Decor[AsyncFn[[], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[QRCodeLoginState.WaitingForConfirm]) -> Decor[AsyncFn[[], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[QRCodeLoginState.Canceled]) -> Decor[AsyncFn[[], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[QRCodeLoginState.Timeout]) -> Decor[AsyncFn[[], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[QRCodeLoginState.Success]) -> Decor[AsyncFn[[int], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[QRCodeLoginState.UINMismatch]) -> Decor[AsyncFn[[int, int], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[QRCodeLoginState.DisplayQRCode]) -> Decor[AsyncFn[[list[list[bool]]], Any]]:
        ...

    def set_handle(self, state) -> Decor[Callable]:
        def register_callback(func: Callable) -> Callable:
            self.callbacks[state] = func
            return func

        return register_callback

    def get_handle(self, state: str) -> Optional[Callable]:
        return self.callbacks.get(QRCodeLoginState(state))

    @classmethod
    def default(cls, qrcode_printer: QRCodeRenderer = Dense1x2(), interval: float = 5.0, merge: bool = True) -> Self:
        cbs = QRCodeLoginCallbacks(interval=interval)
        S = QRCodeLoginState

        last_state: Ref[Optional[S]] = Ref(None)

        def wrap(state: S) -> Callable[[Callable[P, None]], AsyncFn[P, None]]:
            def receiver(func: Callable[P, None]) -> AsyncFn[P, None]:
                import functools

                @functools.wraps(func)
                async def wrapper(*args: P.args, **kwargs: P.kwargs) -> None:
                    if last_state.ref == state and merge:
                        return
                    last_state.ref = state
                    return func(*args, **kwargs)

                return wrapper

            return receiver

        @cbs.set_handle(S.Success)
        @wrap(S.Success)
        def _(uin: int):
            log.success("成功登录账号 {}", uin)

        @cbs.set_handle(S.UINMismatch)
        @wrap(S.UINMismatch)
        def _(uin: int, real_uin: int):
            log.error("预期使用账号 {} 登录，实际登录为 {}", uin, real_uin)
            log.critical("请重新登录")

        @cbs.set_handle(S.DisplayQRCode)
        @wrap(S.DisplayQRCode)
        def _(data: list[list[bool]]):
            log.info("请扫描二维码登录：\n" + qrcode_printer.render(data))

        @cbs.set_handle(S.WaitingForScan)
        @wrap(S.WaitingForScan)
        def _():
            log.debug("等待扫码")

        @cbs.set_handle(S.WaitingForConfirm)
        @wrap(S.WaitingForConfirm)
        def _():
            log.info("扫码成功，等待确认")

        @cbs.set_handle(S.Canceled)
        @wrap(S.Canceled)
        def _():
            log.error("取消扫码，重新尝试登录")

        @cbs.set_handle(S.Timeout)
        @wrap(S.Timeout)
        def _():
            log.error("扫码登录等待超时，尝试重新登录")

        return cbs
