//! 解析和生成 `device.json`。

use anyhow::{anyhow, bail, Result};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use ricq::{device::OSVersion, Device};
use serde_json::{Map, Value};

macro_rules! parse_batch {
    ($version:ty, $json:ident, $fallback:ident, $($key:expr => $name:ident,)*) => {
        Device {
            $($name: <$version>::parse($json, $key, || $fallback.$name.clone())?,)*
        }
    };
}

macro_rules! parse {
    ($version:ty, $json:ident, $fallback:ident) => {
        parse_batch!($version, $json, $fallback,
            "display" => display,
            "product" => product,
            "device" => device,
            "board" => board,
            "model" => model,
            "fingerprint" => finger_print,
            "bootId" => boot_id,
            "procVersion" => proc_version,
            "imei" => imei,
            "brand" => brand,
            "bootloader" => bootloader,
            "baseBand" => base_band,
            "version" => version,
            "simInfo" => sim_info,
            "osType" => os_type,
            "macAddress" => mac_address,
            "ipAddress" => ip_address,
            "wifiBSSID" => wifi_bssid,
            "wifiSSID" => wifi_ssid,
            "imsiMd5" => imsi_md5,
            "androidId" => android_id,
            "apn" => apn,
            "vendorName" => vendor_name,
            "vendorOsName" => vendor_os_name,
        )
    }
}

/// 从 `device.json` 中读取设备信息。
///
/// `device.json` 采用 **mirai 的格式**，与 ricq 的直接定义不兼容。
///
/// # Arguments
/// - `json` - `device.json` 的内容。
/// - `fallback` - 某一项不存在时的默认值。
pub(crate) fn from_json(json: &str, fallback: &Device) -> Result<Device> {
    let json: Value = serde_json::from_str(json)?;
    let json = json
        .as_object()
        .ok_or_else(|| anyhow!("`device.json` 格式错误"))?;
    // 查看版本
    let version = json
        .get("deviceInfoVersion")
        .map(|v| v.as_i64().unwrap_or(-1))
        .unwrap_or(1);
    match version {
        1 => {
            // 版本1：字符串全部使用 UTF-8 字节数组表示，MD5 使用字节数组表示
            Ok(parse!(V1, json, fallback))
        }
        2 => {
            // 版本2：字符串直接储存，MD5 使用十六进制表示
            let json = json
                .get("data")
                .and_then(|v| v.as_object())
                .ok_or_else(|| anyhow!("`device.json` 格式错误"))?;
            Ok(parse!(V2, json, fallback))
        }
        _ => bail!("不支持的版本"),
    }
}

/// 以 QQ 号为种子生成随机的设备信息。
pub(crate) fn random_from_uin(uin: i64) -> Device {
    let mut seed = ChaCha8Rng::seed_from_u64(uin as u64);
    Device::random_with_rng(&mut seed)
}

macro_rules! dump_batch {
    ($json:ident, $device:ident, $($key:expr => $name:ident,)*) => {
        $($json.insert($key.to_string(), V2::dump(&$device.$name));)*
    };
}

macro_rules! dump {
    ($json:ident, $device:ident) => {
        dump_batch!($json, $device,
            "display" => display,
            "product" => product,
            "device" => device,
            "board" => board,
            "model" => model,
            "fingerprint" => finger_print,
            "bootId" => boot_id,
            "procVersion" => proc_version,
            "imei" => imei,
            "brand" => brand,
            "bootloader" => bootloader,
            "baseBand" => base_band,
            "version" => version,
            "simInfo" => sim_info,
            "osType" => os_type,
            "macAddress" => mac_address,
            "ipAddress" => ip_address,
            "wifiBSSID" => wifi_bssid,
            "wifiSSID" => wifi_ssid,
            "imsiMd5" => imsi_md5,
            "androidId" => android_id,
            "apn" => apn,
            "vendorName" => vendor_name,
            "vendorOsName" => vendor_os_name,
        )
    }
}

/// 将设备信息写入 `device.json`。
pub(crate) fn to_json(device: &Device) -> Result<String> {
    let mut json = Map::new();
    json.insert("deviceInfoVersion".into(), Value::Number(2.into()));
    json.insert("data".into(), {
        let mut json = Map::new();
        dump!(json, device);
        json.into()
    });
    Ok(serde_json::to_string_pretty(&json)?)
}

trait Parse<T> {
    fn parse(json: &Map<String, Value>, key: &str, fallback: impl FnOnce() -> T) -> Result<T>;
}

struct V1;

impl Parse<String> for V1 {
    fn parse(
        json: &Map<String, Value>,
        key: &str,
        fallback: impl FnOnce() -> String,
    ) -> Result<String> {
        json.get(key)
            .map(|v| -> Result<String> {
                if let Some(s) = v.as_str() {
                    return Ok(s.to_string());
                }
                let bytes = v
                    .as_array()
                    .ok_or_else(|| anyhow!("`{}` 格式错误", key))?
                    .iter()
                    .map(|b| b.as_i64())
                    .collect::<Option<Vec<i64>>>()
                    .ok_or_else(|| anyhow!("`{}` 格式错误", key))?
                    .iter()
                    .map(|b| b.to_le_bytes()[0])
                    .collect::<Vec<u8>>();
                Ok(String::from_utf8(bytes)?)
            })
            .unwrap_or_else(|| Ok(fallback()))
    }
}

impl Parse<Vec<u8>> for V1 {
    fn parse(
        json: &Map<String, Value>,
        key: &str,
        fallback: impl FnOnce() -> Vec<u8>,
    ) -> Result<Vec<u8>> {
        json.get(key)
            .map(|v| -> Result<Vec<u8>> {
                let bytes = v
                    .as_array()
                    .ok_or_else(|| anyhow!("`{}` 格式错误", key))?
                    .iter()
                    .map(|b| b.as_i64())
                    .collect::<Option<Vec<i64>>>()
                    .ok_or_else(|| anyhow!("`{}` 格式错误", key))?
                    .iter()
                    .map(|b| b.to_le_bytes()[0])
                    .collect::<Vec<u8>>();
                Ok(bytes)
            })
            .unwrap_or_else(|| Ok(fallback()))
    }
}

impl Parse<u32> for V1 {
    fn parse(json: &Map<String, Value>, key: &str, fallback: impl FnOnce() -> u32) -> Result<u32> {
        json.get(key)
            .map(|v| -> Result<u32> {
                let value = v.as_i64().ok_or_else(|| anyhow!("`{}` 格式错误", key))?;
                Ok(value as u32)
            })
            .unwrap_or_else(|| Ok(fallback()))
    }
}

impl Parse<OSVersion> for V1 {
    fn parse(
        json: &Map<String, Value>,
        key: &str,
        fallback: impl FnOnce() -> OSVersion,
    ) -> Result<OSVersion> {
        let version = json
            .get(key)
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow!("`{}` 格式错误", key))?;
        let fallback = fallback();
        let incremental = V1::parse(version, "incremental", || fallback.incremental)?;
        let release = V1::parse(version, "release", || fallback.release)?;
        let codename = V1::parse(version, "codename", || fallback.codename)?;
        let sdk = V1::parse(version, "sdk", || fallback.sdk)?;
        Ok(OSVersion {
            incremental,
            release,
            codename,
            sdk,
        })
    }
}

struct V2;

impl Parse<String> for V2 {
    fn parse(
        json: &Map<String, Value>,
        key: &str,
        fallback: impl FnOnce() -> String,
    ) -> Result<String> {
        json.get(key)
            .map(|v| -> Result<String> {
                Ok(v.as_str()
                    .ok_or_else(|| anyhow!("`{}` 格式错误", key))?
                    .to_string())
            })
            .unwrap_or_else(|| Ok(fallback()))
    }
}

impl Parse<Vec<u8>> for V2 {
    fn parse(
        json: &Map<String, Value>,
        key: &str,
        fallback: impl FnOnce() -> Vec<u8>,
    ) -> Result<Vec<u8>> {
        json.get(key)
            .map(|v| -> Result<Vec<u8>> {
                let hex = v.as_str().ok_or_else(|| anyhow!("`{}` 格式错误", key))?;
                Ok(hex::decode(hex)?)
            })
            .unwrap_or_else(|| Ok(fallback()))
    }
}

impl Parse<u32> for V2 {
    fn parse(json: &Map<String, Value>, key: &str, fallback: impl FnOnce() -> u32) -> Result<u32> {
        json.get(key)
            .map(|v| -> Result<u32> {
                let value = v.as_i64().ok_or_else(|| anyhow!("`{}` 格式错误", key))?;
                Ok(value.try_into()?)
            })
            .unwrap_or_else(|| Ok(fallback()))
    }
}

impl Parse<OSVersion> for V2 {
    fn parse(
        json: &Map<String, Value>,
        key: &str,
        fallback: impl FnOnce() -> OSVersion,
    ) -> Result<OSVersion> {
        let version = json
            .get(key)
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow!("`{}` 格式错误", key))?;
        let fallback = fallback();
        let incremental = V2::parse(version, "incremental", || fallback.incremental)?;
        let release = V2::parse(version, "release", || fallback.release)?;
        let codename = V2::parse(version, "codename", || fallback.codename)?;
        let sdk = V2::parse(version, "sdk", || fallback.sdk)?;
        Ok(OSVersion {
            incremental,
            release,
            codename,
            sdk,
        })
    }
}

trait Dump<T> {
    fn dump(value: &T) -> Value;
}

impl Dump<String> for V2 {
    fn dump(value: &String) -> Value {
        value.to_string().into()
    }
}

impl Dump<Vec<u8>> for V2 {
    fn dump(value: &Vec<u8>) -> Value {
        hex::encode(value).into()
    }
}

impl Dump<u32> for V2 {
    fn dump(value: &u32) -> Value {
        (*value as u64).into()
    }
}

impl Dump<OSVersion> for V2 {
    fn dump(value: &OSVersion) -> Value {
        let mut map = Map::new();
        map.insert("incremental".to_string(), V2::dump(&value.incremental));
        map.insert("release".to_string(), V2::dump(&value.release));
        map.insert("codename".to_string(), V2::dump(&value.codename));
        map.insert("sdk".to_string(), V2::dump(&value.sdk));
        map.into()
    }
}
