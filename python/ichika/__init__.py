from __future__ import annotations
from . import ichika as lib
from .stubs import _LoginMethodTransfer

lib.init_log(lib)

__version__ = lib.__version__
__build__ = lib.__build__
Account = lib.Account


class LoginMethod:
    QRCode = _LoginMethodTransfer("""{"type": "QRCode"}""")

    @staticmethod
    def Password(
        password: str, md5: bool = False, sms: bool = False
    ) -> _LoginMethodTransfer:
        import json

        return _LoginMethodTransfer(
            json.dumps(
                {"type": "Password", "password": password, "md5": md5, "sms": sms}
            )
        )
