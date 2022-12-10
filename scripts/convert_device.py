"""转换 device.json 为 ricq 使用的格式"""
import argparse
import hashlib
import json
import os
import string
import sys
import uuid
from dataclasses import asdict as to_dict
from dataclasses import dataclass, field
from random import Random
from typing import Union


@dataclass
class OSVersion:
    incremental: str = "5891938"
    release: str = "10"
    codename: str = "REL"
    sdk: int = 29


rng = Random()


def random_imei() -> str:
    tot: int = 0
    res: list[str] = []
    for i in range(15):
        to_add = rng.randrange(0, 10)
        if (i + 2) % 2 == 0:
            to_add *= 2
            if to_add >= 10:
                to_add = (to_add % 10) + 1
        tot += to_add
        res.append(str(to_add))
    res.append(str(tot * 9 % 10))
    return "".join(res)


@dataclass
class RICQDevice:
    display: str = field(default_factory=lambda: f"RICQ.{rng.randrange(100000, 1000000)}.001")
    product: str = "iarim"
    device: str = "sagit"
    board: str = "eomam"
    model: str = "MI 6"
    finger_print: str = field(default_factory=lambda: f"RICQ.{rng.randrange(100000, 1000000)}.001")
    boot_id: str = field(default=str(uuid.uuid4()))
    proc_version: str = field(
        default_factory=lambda: f"Linux 5.4.0-54-generic-{''.join(rng.choices(string.hexdigits, k=8))} (android-build@google.com)"
    )
    imei: str = field(default_factory=random_imei)
    brand: str = "Xiaomi"
    bootloader: str = "U-boot"
    base_band: str = ""
    version: OSVersion = field(default_factory=OSVersion)
    sim_info: str = "T-Mobile"
    os_type: str = "android"
    mac_address: str = "00:50:56:C0:00:08"
    ip_address: list[int] = field(default_factory=lambda: [10, 0, 1, 3])
    wifi_bssid: str = "02:00:00:00:00:00"
    wifi_ssid: str = "<unknown ssid>"
    imsi_md5: list[int] = field(default_factory=lambda: list(hashlib.md5(rng.randbytes(16)).digest()))
    android_id: str = field(default_factory=lambda: "".join(f"{t:02x}" for t in rng.randbytes(8)))
    apn: str = "wifi"
    vendor_name: str = "MIUI"
    vendor_os_name: str = "ricq"


def string_hook(source: Union[str, list[int]]) -> str:
    return source if isinstance(source, str) else bytes(source).decode("utf-8")


def list_int_hook(source: Union[list[int], str]) -> list[int]:
    return [t % 256 for t in source] if isinstance(source, list) else list(bytes.fromhex(source))


def camel_to_snake(src: str) -> str:
    return "".join([f"_{i.lower()}" if i.isupper() else i for i in src]).lstrip("_")


import contextlib

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("in_file", help="输入路径")
    parser.add_argument(
        "out_file",
        nargs="?",
        default="device_ricq.json",
        help="输出路径（单个横杠表示输出到 stdout）",
    )
    nbsp = parser.parse_args()
    in_file: str = nbsp.in_file
    out_file: str = nbsp.out_file
    with open(in_file, "r", encoding="utf-8") as src_file:
        source: dict = json.loads(src_file.read())
        if "deviceInfoVersion" in source:
            assert source["deviceInfoVersion"] in (1, 2), "不支持的 device info 版本！"
            source = source["data"]
        if "fingerprint" in source:
            source["finger_print"] = source["fingerprint"]
        source.update({camel_to_snake(k): source[k] for k in source})
    try:
        import dacite.core
        from dacite.config import Config
    except ImportError as err:
        raise ImportError("请安装 dacite") from err
    device = dacite.core.from_dict(RICQDevice, source, Config({str: string_hook, list[int]: list_int_hook}))
    result = json.dumps(to_dict(device), indent=4)
    if out_file == "-":
        with contextlib.suppress(ImportError):
            from rich import print
        print(result)
    else:
        if os.path.exists(out_file):
            choice = input(f"警告：{out_file} 已存在，是否覆盖？(y/N)").lower()
            if choice != "y":
                sys.exit(0)
        with open(out_file, "w") as out:
            out.write(result)
