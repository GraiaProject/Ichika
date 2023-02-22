from __future__ import annotations

import json
import os
from enum import Enum, auto
from pathlib import Path
from typing import (
    Any,
    Callable,
    Literal,
    NoReturn,
    Optional,
    Self,
    TypeVar,
    Union,
    overload,
)

from loguru import logger as log

import ichika.core as _core
from ichika.client import Client
from ichika.login.qrcode_render import Dense1x2, QRCodeRenderer


class Protocol(str, Enum):
    IPad = "IPad"
    AndroidPhone = "AndroidPhone"
    AndroidWatch = "AndroidWatch"
    MacOS = "MacOS"
    QiDian = "QiDian"


_C_T = TypeVar("_C_T", bound=Callable)
_CBDecor = Callable[[_C_T], _C_T]


class AutoName(Enum):
    def _generate_next_value_(name, *_):
        return name


# TODO: reorganize package
# TODO: split Password related and QRCode related


class PasswordLoginState(str, AutoName):
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
    def set_handle(self, state: Literal[PasswordLoginState.DeviceLocked]) -> _CBDecor[Callable[[str, str], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[PasswordLoginState.RequestSMS]) -> _CBDecor[Callable[[str, str], str]]:
        ...

    @overload
    def set_handle(self, state: Literal[PasswordLoginState.NeedCaptcha]) -> _CBDecor[Callable[[str], str]]:
        ...

    @overload
    def set_handle(self, state: Literal[PasswordLoginState.Success]) -> _CBDecor[Callable[[], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[PasswordLoginState.DeviceLockLogin]) -> _CBDecor[Callable[[], Any]]:
        ...

    @overload
    def set_handle(
        self,
        state: Literal[PasswordLoginState.AccountFrozen],
    ) -> _CBDecor[Callable[[], NoReturn]]:
        ...

    @overload
    def set_handle(
        self,
        state: Literal[PasswordLoginState.TooManySMSRequest],
    ) -> _CBDecor[Callable[[], NoReturn]]:
        ...

    @overload
    def set_handle(
        self,
        state: Literal[PasswordLoginState.UnknownStatus],
    ) -> _CBDecor[Callable[[str, int], NoReturn]]:
        ...

    def set_handle(self, state) -> _CBDecor[Callable]:
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
        def _(url: str):
            log.warning(f"请完成滑块验证，URL: {url}")
            return input("完成后请输入 ticket >").strip(" ")

        @cbs.set_handle(S.DeviceLocked)
        def _(message: str, url: str):
            log.warning(message)
            log.warning(f"请完成设备锁验证，URL: {url}")
            input("请在完成后回车")

        @cbs.set_handle(S.RequestSMS)
        def _(message: str, phone_number: str) -> str:
            log.warning(message)
            log.warning(f"已发送短信验证码至 {phone_number}")
            return input("请输入收到的短信验证码 >").strip(" ")

        @cbs.set_handle(S.AccountFrozen)
        def _() -> NoReturn:
            msg = "无法登录：账号被冻结"
            raise RuntimeError(msg)

        @cbs.set_handle(S.TooManySMSRequest)
        def _() -> NoReturn:
            msg = "短信请求次数过多，请稍后再试"
            raise RuntimeError(msg)

        @cbs.set_handle(S.UnknownStatus)
        def _(message: str, code: int) -> NoReturn:
            msg = f"未知错误（代码 {code}）：{message}"
            raise RuntimeError(msg)

        cbs.set_handle(S.Success)(lambda: log.success("登录成功"))
        cbs.set_handle(S.DeviceLockLogin)(lambda: log.info("尝试设备锁登录"))
        return cbs


class QRCodeLoginState(str, AutoName):
    WaitingForScan = auto()
    WaitingForConfirm = auto()
    Canceled = auto()
    Timeout = auto()
    Success = auto()
    DisplayQRCode = auto()
    UINMismatch = auto()


class QRCodeLoginCallbacks:
    def __init__(self, callbacks: dict[QRCodeLoginState, Callable] | None = None):
        self.callbacks: dict[QRCodeLoginState, Optional[Callable]] = {state: None for state in QRCodeLoginState}
        self.callbacks.update(callbacks or {})

    @overload
    def set_handle(self, state: Literal[QRCodeLoginState.WaitingForScan]) -> _CBDecor[Callable[[], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[QRCodeLoginState.WaitingForConfirm]) -> _CBDecor[Callable[[], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[QRCodeLoginState.Canceled]) -> _CBDecor[Callable[[], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[QRCodeLoginState.Timeout]) -> _CBDecor[Callable[[], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[QRCodeLoginState.Success]) -> _CBDecor[Callable[[int], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[QRCodeLoginState.UINMismatch]) -> _CBDecor[Callable[[int, int], Any]]:
        ...

    @overload
    def set_handle(self, state: Literal[QRCodeLoginState.DisplayQRCode]) -> _CBDecor[Callable[[list[list[bool]]], Any]]:
        ...

    def set_handle(self, state) -> _CBDecor[Callable]:
        def register_callback(func: Callable) -> Callable:
            self.callbacks[state] = func
            return func

        return register_callback

    def get_handle(self, state: str) -> Optional[Callable]:
        return self.callbacks.get(QRCodeLoginState(state))

    @classmethod
    def default(cls, qrcode_printer: QRCodeRenderer = Dense1x2()) -> Self:
        cbs = QRCodeLoginCallbacks()
        S = QRCodeLoginState

        # TODO: support elegant state transition
        @cbs.set_handle(S.Success)
        def _(uin: int):
            log.success("成功登录账号 {}", uin)

        @cbs.set_handle(S.UINMismatch)
        def _(uin: int, real_uin: int):
            log.error("预期使用账号 {} 登录，实际登录为 {}", uin, real_uin)
            log.critical("请重新登录")

        @cbs.set_handle(S.DisplayQRCode)
        def _(data: list[list[bool]]):
            log.info("请扫描二维码登录：\n" + qrcode_printer.render(data))

        cbs.set_handle(S.WaitingForScan)(lambda: log.debug("等待扫码"))
        cbs.set_handle(S.WaitingForConfirm)(lambda: log.info("扫码成功，等待确认"))
        cbs.set_handle(S.Canceled)(lambda: log.error("取消扫码，重新尝试登录"))
        cbs.set_handle(S.Timeout)(lambda: log.error("扫码登录等待超时，尝试重新登录"))
        return cbs


class BaseLoginCredentialStore:
    def get_token(self, uin: int, protocol: Protocol) -> Optional[bytes]:
        pass

    def write_token(self, uin: int, protocol: Protocol, token: bytes) -> None:
        pass

    def get_device(self, uin: int, protocol: Protocol) -> dict:
        from dataclasses import asdict
        from random import Random

        from ichika.scripts.device.generator import generate

        return asdict(generate(Random(hash(protocol) ^ uin)))


class PathCredentialStore(BaseLoginCredentialStore):
    """可以给所有账号共享的，基于路径的凭据存储器"""

    def __init__(self, path: Union[str, os.PathLike[str]]) -> None:
        self.path = Path(path)
        self.path.mkdir(parents=True, exist_ok=True)

    def uin_path(self, uin: int) -> Path:
        path = self.path / str(uin)
        path.mkdir(parents=True, exist_ok=True)
        return path

    def get_device(self, uin: int, protocol: Protocol) -> dict:
        ricq_device = self.uin_path(uin) / "ricq_device.json"
        if ricq_device.exists():
            log.info("发现 `ricq_device.json`, 读取")
            return json.loads(ricq_device.read_text("utf-8"))

        other_device = self.uin_path(uin) / "device.json"
        if other_device.exists():
            from dataclasses import asdict

            from ichika.scripts.device.converter import convert

            log.info("发现其他格式的 `device.json`, 尝试转换")
            device_content = asdict(convert(json.loads(other_device.read_text("utf-8"))))
        else:
            log.info("未发现 `device.json`, 正在生成")
            device_content = super().get_device(uin, protocol)

        ricq_device.write_text(json.dumps(device_content, indent=4), "utf-8")
        return device_content

    def get_token(self, uin: int, protocol: Protocol) -> Optional[bytes]:
        token = self.uin_path(uin) / f"token-{protocol.value}.bin"
        return token.read_bytes() if token.exists() else None

    def write_token(self, uin: int, protocol: Protocol, token: bytes) -> None:
        token_path = self.uin_path(uin) / f"token-{protocol.value}.bin"
        token_path.write_bytes(token)


@overload
async def login_password(
    uin: int,
    password: str,
    /,
    protocol: Protocol,
    store: BaseLoginCredentialStore,
    event_callbacks: list[Callable[[Any], Any]],
    callbacks: PasswordLoginCallbacks = ...,
    use_sms: bool = ...,
) -> Client:
    ...


@overload
async def login_password(
    uin: int,
    password_md5: bytes,
    /,
    protocol: Protocol,
    store: BaseLoginCredentialStore,
    event_callbacks: list[Callable[[Any], Any]],
    callbacks: PasswordLoginCallbacks = ...,
    use_sms: bool = ...,
) -> Client:
    ...


async def login_password(
    uin: int,
    credential: Union[str, bytes],
    /,
    protocol: Protocol,
    store: BaseLoginCredentialStore,
    event_callbacks: list[Callable[[Any], Any]],
    callbacks: PasswordLoginCallbacks = PasswordLoginCallbacks.default(),
    use_sms: bool = True,
) -> Client:
    return await _core.password_login(uin, credential, use_sms, protocol, store, event_callbacks, callbacks)


async def login_qrcode(
    uin: int,
    /,
    protocol: Literal[Protocol.AndroidWatch, Protocol.MacOS],
    store: BaseLoginCredentialStore,
    event_callbacks: list[Callable[[Any], Any]],
    callbacks: QRCodeLoginCallbacks = QRCodeLoginCallbacks.default(),
) -> Client:
    return await _core.qrcode_login(uin, protocol, store, event_callbacks, callbacks)
    # TODO: support poll interval
