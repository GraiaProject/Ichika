from __future__ import annotations

import hashlib
import string
import uuid
from dataclasses import asdict as to_dict
from random import Random
from typing import Any, List

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
    from .generator import generate

    return to_dict(generate())


def convert(source: dict) -> RICQDevice:
    if "deviceInfoVersion" in source:  # mirai
        source = source["data"]
    if "fingerprint" in source:
        source["finger_print"] = source["fingerprint"]
    params = make_defaults()
    version: Any = source.setdefault("version", params["version"])
    version.update(source.get("version", {}))
    for key in source:
        converted_key = camel_to_snake(key)
        if converted_key in params:
            if isinstance(params[converted_key], dict):
                params[converted_key].update(source[key])
            else:
                params[converted_key] = source[key]
    return from_dict(RICQDevice, params, Config({str: string_hook, List[int]: list_int_hook}))
