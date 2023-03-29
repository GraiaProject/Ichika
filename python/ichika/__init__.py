from __future__ import annotations

from . import core as core
from .build_info import BuildInfo

__version__: str = core.__version__
"""Ichika 当前版本号"""

__build__: BuildInfo = core.__build__
"""Ichika 的构建信息"""
