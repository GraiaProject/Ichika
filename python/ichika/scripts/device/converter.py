from __future__ import annotations

import hashlib
import string
import uuid
from dataclasses import asdict as to_dict
from random import Random

from dacite.config import Config
from dacite.core import from_dict

from . import RICQDevice, data


def string_hook(source: str | list[int]) -> str:
    return source if isinstance(source, str) else bytes(source).decode("utf-8")


def list_int_hook(source: str | list[int]) -> list[int]:
    return [t % 256 for t in source] if isinstance(source, list) else list(bytes.fromhex(source))


def camel_to_snake(src: str) -> str:
    return "".join([f"_{i.lower()}" if i.isupper() else i for i in src]).lstrip("_")


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


def make_defaults() -> dict:
    return {
        "display": f"RICQ.{rng.randrange(100000, 1000000)}.001",
        "product": "iarim",
        "device": "sagit",
        "board": "eomam",
        "model": "MI 6",
        "finger_print": f"RICQ.{rng.randrange(100000, 1000000)}.001",
        "boot_id": str(uuid.uuid4()),
        "proc_version": f"Linux 5.4.0-54-generic-{''.join(rng.choices(string.hexdigits, k=8))} (android-build@google.com)",
        "imei": random_imei(),
        "brand": "Xiaomi",
        "bootloader": "U-boot",
        "base_band": "",
        "version": to_dict(rng.choice(data.os_versions)),
        "sim_info": "T-Mobile",
        "os_type": "android",
        "mac_address": "00:50:56:C0:00:08",
        "ip_address": [10, 0, 1, 3],
        "wifi_bssid": "02:00:00:00:00:00",
        "wifi_ssid": "<unknown ssid>",
        "imsi_md5": list(hashlib.md5(rng.randbytes(16)).digest()),
        "android_id": "".join(f"{t:02x}" for t in rng.randbytes(8)),
        "apn": "wifi",
        "vendor_name": "MIUI",
        "vendor_os_name": "ricq",
    }


def convert(source: dict) -> RICQDevice:
    if "deviceInfoVersion" in source:  # mirai
        source = source["data"]
    if "fingerprint" in source:
        source["finger_print"] = source["fingerprint"]
    params = make_defaults()
    source.setdefault("version", params["version"]).update(source.get("version", {}))
    params.update({camel_to_snake(k): source[k] for k in source})
    return from_dict(RICQDevice, params, Config({str: string_hook, list[int]: list_int_hook}))
