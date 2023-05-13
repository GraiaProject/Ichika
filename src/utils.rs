use futures_util::Future;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

// use pyo3::types::*;

/// 获取 Python 的 None。
pub fn py_none() -> PyObject {
    py_use(|py| py.None())
}

pub trait AsPython {
    fn obj(self) -> PyObject;
}

impl<T> AsPython for T
where
    T: IntoPy<PyObject>,
{
    fn obj(self) -> PyObject {
        py_use(|py| self.into_py(py))
    }
}

pub fn py_bytes(data: &[u8]) -> Py<PyBytes> {
    py_use(|py| PyBytes::new(py, data).into_py(py))
}

/// 构造一个 Python 的 dict。
#[macro_export]
#[doc(hidden)]
macro_rules! py_dict {
    ($py:expr, $($name:expr => $value:expr),*) => {
        {
            let dict = ::pyo3::types::PyDict::new($py);
            $(
                dict.set_item($name, $value).expect("Failed to set_item on dict");
            )*
            dict
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! dict {
    {$py:expr, $($name:ident : $value:expr),* $(,)?} => {
        {
            let dict = ::pyo3::types::PyDict::new($py);
            $(
                dict.set_item(stringify!($name), $value)?;
            )*
            dict
        }
    };
}

#[macro_export]
macro_rules! import_call {
    ($py:expr, $module:expr => $attr:expr => $arg:expr) => {
        $py.import(::pyo3::intern!($py, $module))?
            .getattr(::pyo3::intern!($py, $attr))?
            .call1(($arg,))
    };
    ($py:expr, $module:expr => $attr:expr => @tuple $arg:expr) => {
        $py.import(::pyo3::intern!($py, $module))?
            .getattr(::pyo3::intern!($py, $attr))?
            .call1($arg)
    };
}

#[macro_export]
macro_rules! props {
    ($self_t:ident @ $cls:ident : $($name:ident => [$type:ty] $res:stmt);* ;) => {
        #[::pyo3::pymethods]
        impl $cls {
            $(
                #[getter]
                pub fn $name(&$self_t) -> $type {
                    $res
                }
            )*
        }
    };
}

/// 将 [`tokio`] 的 Future 包装为 Python 的 Future。
pub fn py_future<F, T>(py: Python, future: F) -> PyResult<&PyAny>
where
    F: Future<Output = Result<T, crate::exc::Error>> + Send + 'static,
    T: IntoPy<PyObject>,
{
    pyo3_asyncio::tokio::future_into_py(py, async move { future.await.map_err(|e| e.into()) })
}

pub fn py_try<F, R>(f: F) -> PyResult<R>
where
    F: for<'py> FnOnce(Python<'py>) -> PyResult<R>,
{
    Python::with_gil(f)
}

pub fn py_use<F, R>(f: F) -> R
where
    F: for<'py> FnOnce(Python<'py>) -> R,
{
    Python::with_gil(f)
}

#[macro_export]
macro_rules! static_py_fn {
    ($name:ident, $cell_name:ident, $module:expr, [$($attr:expr),*]) => {
        #[allow(non_upper_case_globals, reason = "Not controllable via declarative macros")]
        static $cell_name: ::pyo3::once_cell::GILOnceCell<PyObject> = ::pyo3::once_cell::GILOnceCell::new();

        pub fn $name(python: ::pyo3::marker::Python) -> &pyo3::PyAny {
            $cell_name.get_or_init(python, || {
                python
                .import(::pyo3::intern!(python, $module)).expect(concat!("Unable to import module ", $module))
                $(.getattr(::pyo3::intern!(python, $attr)).expect(concat!("Unable to get attribute ", $attr)))*
                .into()
                }
            )
            .as_ref(python)
        }
    };
}

#[macro_export]
macro_rules! call_static_py {
    ($pth:expr, $py:expr, ($($arg:expr),*)) => {
        $pth($py).call1(
            ($($arg,)*)
        )
    };
    ($pth:expr, $py:expr, ($($arg:expr),*) ! $reason:expr) => {
        $pth($py).call1(
            ($($arg,)*)
        )
        .expect($reason)
        .into()
    }
}

static_py_fn!(
    _datetime_from_ts,
    __DT_CELL,
    "datetime",
    ["datetime", "fromtimestamp"]
);

pub fn datetime_from_ts(py: Python<'_>, time: impl IntoPy<PyObject>) -> PyResult<&PyAny> {
    call_static_py!(_datetime_from_ts, py, (time))
}

static_py_fn!(
    _timedelta_from_secs,
    __TDELTA_CELL,
    "datetime",
    ["timedelta"]
);

pub fn timedelta_from_secs(py: Python<'_>, delta: impl IntoPy<PyObject>) -> PyResult<&PyAny> {
    _timedelta_from_secs(py).call((), Some(dict!(py, seconds: delta.into_py(py))))
}

static_py_fn!(partial, __PARTIAL_CELL, "functools", ["partial"]);

static_py_fn!(
    py_client_refs,
    __CLIENT_WEAKREFS_CELL,
    "ichika.client",
    ["CLIENT_REFS"]
);

static_py_fn!(
    _to_py_gender,
    __PY_GENDER_ENUM_CELL,
    "ichika.structs",
    ["Gender"]
);

pub fn to_py_gender(gender: u8) -> PyObject {
    let gender_str = match gender {
        0 => "Male",
        1 => "Female",
        _ => "Unknown",
    };
    py_use(|py| _to_py_gender(py).call1((gender_str,)).unwrap().into_py(py))
}

static_py_fn!(
    _to_py_perm,
    __PY_GROUP_PERMISSION_CELL,
    "ichika.structs",
    ["GroupPermission"]
);

pub fn to_py_permission(perm: ricq_core::structs::GroupMemberPermission) -> PyObject {
    use ricq_core::structs::GroupMemberPermission as Perm;
    let perm_str = match perm {
        Perm::Owner => "Owner",
        Perm::Administrator => "Admin",
        Perm::Member => "Member",
    };
    py_use(|py| _to_py_perm(py).call1((perm_str,)).unwrap().into_py(py))
}

#[macro_export]
macro_rules! dict_obj {
    {$py:ident ! $($key:ident : $val:expr),* $(,)?} => {
        ::pyo3::Python::with_gil(|$py| -> ::pyo3::PyResult<_> {
            let dict = ::pyo3::types::PyDict::new($py);
            $(
                let _val: ::pyo3::PyObject = $val.into_py($py);
                dict.set_item(stringify!($key), _val)?;
            )*
            Ok(dict.into_py($py))
        })
    };
    {$($key:ident : $val:expr),* $(,)?} => {
        dict_obj!(py ! $($key : $val),*)
    }
}
