import importlib.resources
import json
from dataclasses import dataclass
from typing import Dict, List, Optional

from dacite.core import from_dict


@dataclass
class OSVersion:
    incremental: str
    release: str
    codename: str
    sdk: int


@dataclass
class RICQDevice:
    display: str
    product: str
    device: str
    board: str
    model: str
    finger_print: str
    boot_id: str
    proc_version: str
    imei: str
    brand: str
    bootloader: str
    base_band: str
    version: OSVersion
    sim_info: str
    os_type: str
    mac_address: str
    ip_address: List[int]
    wifi_bssid: str
    wifi_ssid: str
    imsi_md5: List[int]
    android_id: str
    apn: str
    vendor_name: str
    vendor_os_name: str


@dataclass
class Model:
    name: str
    brand: str
    tac: str
    fac: str
    board: str
    device: str
    display: str
    proc: Optional[str] = None
    os_versions: Optional[List[OSVersion]] = None
    model: Optional[str] = None
    finger: Optional[str] = None


@dataclass
class Data:
    os_versions: List[OSVersion]
    addr: Dict[str, List[str]]
    models: List[Model]


data: Data = from_dict(Data, json.loads(importlib.resources.read_text(__name__, "data.json", "utf-8")))
