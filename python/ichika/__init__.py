from __future__ import annotations

from typing import Any

from . import ichika as lib

lib.init_log(lib)

__version__ = lib.__version__
__build__ = lib.__build__
Account = lib.Account


class LoginMethod:
    QRCode = {"type": "QRCode"}

    @staticmethod
    def Password(password: str, md5: bool = False, sms: bool = False) -> dict[str, Any]:

        return {"type": "Password", "password": password, "md5": md5, "sms": sms}
