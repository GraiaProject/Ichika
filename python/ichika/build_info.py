import datetime
from typing import TypedDict


class RustCInfo(TypedDict):
    """编译器信息"""

    rustc: str
    """RustC 名称"""
    rustc_version: str
    """RustC 版本"""
    opt_level: str
    """优化等级"""
    debug: bool
    """是否为调试编译"""
    jobs: int
    """编译并行数"""


class TargetInfo(TypedDict):
    """编译目标信息"""

    arch: str
    """目标架构"""
    os: str
    """目标操作系统"""
    family: str
    """目标家族"""
    compiler: str
    """使用的编译器"""
    triple: str
    """目标架构标识"""
    endian: str
    """目标端序"""
    pointer_width: str
    """目标指针宽度"""
    profile: str
    """使用的配置"""


class BuildInfo(TypedDict):
    """Ichika 构建信息"""

    builder: RustCInfo
    """Rust 编译器信息"""
    target: TargetInfo
    """编译目标信息"""
    build_time: datetime.datetime
    """构建时间"""
    dependencies: dict
    """构建依赖字典"""
    host_triple: str
    """编译器的架构标识"""
