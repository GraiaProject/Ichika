use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::dict;
include!(concat!(env!("OUT_DIR"), "/build-info.rs"));

pub fn get_info(py: Python) -> PyResult<&PyDict> {
    let builder = dict! {py,
        rustc: RUSTC,
        rustc_version: RUSTC_VERSION,
        opt_level: OPT_LEVEL,
        debug: DEBUG,
        jobs: NUM_JOBS,
    };

    let target = dict! {py,
        arch: CFG_TARGET_ARCH,
        os: CFG_OS,
        family: CFG_FAMILY,
        compiler: CFG_ENV,
        triple: TARGET,
        endian: CFG_ENDIAN,
        pointer_width: CFG_POINTER_WIDTH,
        profile: PROFILE,
    };

    let dependencies = PyDict::new(py);
    for (name, ver) in DEPENDENCIES {
        dependencies.set_item(name, ver)?;
    }

    let build_time = py
        .import("email.utils")?
        .getattr("parsedate_to_datetime")?
        .call1((BUILT_TIME_UTC,))?
        .into_py(py);

    Ok(dict! {py,
        builder: builder,
        target: target,
        dependencies: dependencies,
        build_time: build_time,
    })
}
