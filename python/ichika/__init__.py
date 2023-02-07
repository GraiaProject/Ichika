from __future__ import annotations

from typing import Any

from . import core as core

core.init_log(core)

__version__ = core.__version__
__build__ = core.__build__
Account = core.Account


class LoginMethod:
    QRCode = {"type": "QRCode"}

    @staticmethod
    def Password(password: str, md5: bool = False, sms: bool = False) -> dict[str, Any]:
        return {"type": "Password", "password": password, "md5": md5, "sms": sms}
