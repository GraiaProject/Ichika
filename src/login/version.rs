use pyo3::exceptions::PyValueError;
use pyo3::PyResult;
use ricq::version::{Protocol, Version};

pub const ANDROID_PHONE: Version = Version {
    apk_id: "com.tencent.mobileqq",
    app_id: 537153294,
    sub_app_id: 537153294,
    sort_version_name: "8.9.35",
    build_ver: "8.9.35.10440",
    build_time: 1676531414,
    apk_sign: &[
        0xA6, 0xB7, 0x45, 0xBF, 0x24, 0xA2, 0xC2, 0x77, 0x52, 0x77, 0x16, 0xF6, 0xF3, 0x6E, 0xB6,
        0x8D,
    ],
    sdk_version: "6.0.0.2535",
    sso_version: 19,
    misc_bitmap: 150470524,
    sub_sig_map: 66560,
    main_sig_map: 16724722,
    protocol: Protocol::AndroidPhone,
};

pub const IPAD: Version = Version {
    apk_id: "com.tencent.minihd.qq",
    app_id: 537157363,
    sub_app_id: 537157363,
    sort_version_name: "8.9.33",
    build_ver: "8.9.33.614",
    build_time: 1640921786,
    apk_sign: &[
        0xAA, 0x39, 0x78, 0xF4, 0x1F, 0xD9, 0x6F, 0xF9, 0x91, 0x4A, 0x66, 0x9E, 0x18, 0x64, 0x74,
        0xC7,
    ],
    sdk_version: "6.0.0.2433",
    sso_version: 12,
    misc_bitmap: 150470524,
    sub_sig_map: 66560,
    main_sig_map: 1970400,
    protocol: Protocol::AndroidPhone,
};

pub const ANDROID_WATCH: Version = Version {
    apk_id: "com.tencent.qqlite",
    app_id: 537065138,
    sub_app_id: 537065138,
    sort_version_name: "2.0.8",
    build_ver: "2.0.8",
    build_time: 1559564731,
    apk_sign: &[
        0xA6, 0xB7, 0x45, 0xBF, 0x24, 0xA2, 0xC2, 0x77, 0x52, 0x77, 0x16, 0xF6, 0xF3, 0x6E, 0xB6,
        0x8D,
    ],
    sdk_version: "6.0.0.2365",
    sso_version: 5,
    misc_bitmap: 16252796,
    sub_sig_map: 0x10400,
    main_sig_map: 16724722,
    protocol: Protocol::AndroidPhone,
};

pub const ANDROID_PAD: Version = Version {
    apk_id: "com.tencent.qqlite",
    app_id: 537152242,
    sub_app_id: 537152242,
    sort_version_name: "8.9.35.10440",
    build_ver: "8.9.35.10440",
    build_time: 1676531414,
    apk_sign: &[
        0xA6, 0xB7, 0x45, 0xBF, 0x24, 0xA2, 0xC2, 0x77, 0x52, 0x77, 0x16, 0xF6, 0xF3, 0x6E, 0xB6,
        0x8D,
    ],
    sdk_version: "6.0.0.253",
    sso_version: 19,
    misc_bitmap: 150470524,
    sub_sig_map: 66560,
    main_sig_map: 150470524,
    protocol: Protocol::AndroidPhone,
};

pub const MACOS: Version = Version {
    apk_id: "com.tencent.qq",              // ok
    app_id: 0x2003ca32,                    // ok
    sub_app_id: 0x2003ca32,                // ok
    sort_version_name: "6.7.9",            // ok
    build_ver: "5.8.9.3460",               // 6.7.9.xxx?
    build_time: 0,                         // ok
    apk_sign: "com.tencent.qq".as_bytes(), // ok
    sdk_version: "6.2.0.1023",             // ok
    sso_version: 7,                        // ok
    misc_bitmap: 0x7ffc,                   // ok
    sub_sig_map: 66560,                    // ?
    main_sig_map: 1970400,                 // ?
    protocol: Protocol::MacOS,
};

pub const QIDIAN: Version = Version {
    apk_id: "com.tencent.qidian",
    app_id: 537061386,
    sub_app_id: 537036590,
    sort_version_name: "3.8.6",
    build_ver: "8.8.38.2266",
    build_time: 1556628836,
    apk_sign: &[
        160, 30, 236, 171, 133, 233, 227, 186, 43, 15, 106, 21, 140, 133, 92, 41,
    ],
    sdk_version: "6.0.0.2365",
    sso_version: 5,
    misc_bitmap: 49807228,
    sub_sig_map: 66560,
    main_sig_map: 34869472,
    protocol: Protocol::QiDian,
};

pub fn get_version(p: String) -> PyResult<Version> {
    match p.as_str() {
        "IPad" => Ok(IPAD),
        "AndroidPhone" => Ok(ANDROID_PHONE),
        "AndroidWatch" => Ok(ANDROID_WATCH),
        "AndroidPad" => Ok(ANDROID_PAD),
        "MacOS" => Ok(MACOS),
        "QiDian" => Ok(QIDIAN),
        _ => Err(PyValueError::new_err("未知协议")),
    }
}
