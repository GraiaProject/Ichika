class IchikaError(Exception):
    """Ichika 所有异常的基类"""


class LoginError(IchikaError, ValueError):
    """登录时因为用户操作不正确引发的异常"""


class RICQError(IchikaError, RuntimeError):
    """由 RICQ 引发的运行时异常"""
