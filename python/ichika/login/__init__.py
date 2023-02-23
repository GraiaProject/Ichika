from __future__ import annotations

import json
import os
from enum import Enum
from pathlib import Path
from typing import Any, Callable, Literal, Optional, Union, overload

from loguru import logger as log

import ichika.core as _core
from ichika.client import Client

from .password import PasswordLoginCallbacks as PasswordLoginCallbacks
from .qrcode import QRCodeLoginCallbacks as QRCodeLoginCallbacks


class Protocol(str, Enum):
    IPad = "IPad"
    AndroidPhone = "AndroidPhone"
    AndroidWatch = "AndroidWatch"
    MacOS = "MacOS"
    QiDian = "QiDian"


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
