import hashlib
import string
import uuid
from random import Random

from . import Model, OSVersion, RICQDevice, data


def gen_finger_print(model: Model, version: OSVersion) -> str:
    device: str = model.model or model.device
    return f"{model.brand}/{device}/{device}:{version.release}/{model.display}/{version.incremental}:user/release-keys"


def luhn(code: str) -> int:
    tot: int = 0

    def parse_even(p: int) -> int:
        return p % 10 + p // 10

    for i in range(len(code)):
        tot += int(code[i]) if i % 2 == 0 else parse_even(int(code[i]) * 2)
    return tot * 9 % 10


def get_imei(rng: Random, model: Model) -> str:
    snr = str(rng.randrange(100000, 1000000))
    sp = luhn(model.tac + model.fac + snr)
    return model.tac + model.fac + snr + str(sp)


def get_mac_addr(rng: Random, model: Model) -> str:
    if model.brand in data.addr:
        return rng.choice(data.addr[model.brand]) + "".join(f":{rng.randrange(0, 256):02x}" for _ in range(3))
    return ":".join(f"{rng.randrange(0, 256):02x}" for _ in range(6))


def generate(rng: Random = Random(hash(""))) -> RICQDevice:
    model = rng.choice(data.models)
    os_version: OSVersion = rng.choice(model.os_versions) if model.os_versions else rng.choice(data.os_versions)
    return RICQDevice(
        display=model.display,
        product=model.name,
        device=model.device,
        board=model.board,
        brand=model.brand,
        model=model.model or model.device,
        bootloader="unknown",
        proc_version=model.proc
        or f"Linux 5.4.0-54-generic-{''.join(rng.choices(string.hexdigits, k=8))} (android-build@google.com)",
        base_band="",
        finger_print=gen_finger_print(model, os_version),
        boot_id=str(uuid.uuid4()),
        imei=get_imei(rng, model),
        version=os_version,
        sim_info="T-Mobile",
        os_type="android",
        wifi_bssid="02:00:00:00:00:00",
        wifi_ssid="<unknown ssid>",
        imsi_md5=list(hashlib.md5(rng.getrandbits(16 * 8).to_bytes(16, "little")).digest()),
        ip_address=[10, 0, 1, 3],
        apn="wifi",
        mac_address=get_mac_addr(rng, model),
        android_id="".join(f"{rng.randrange(0, 256):02x}" for _ in range(8)),
        vendor_name=model.brand.lower(),
        vendor_os_name="unknown",
    )
