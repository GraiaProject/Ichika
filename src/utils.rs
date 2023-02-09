use anyhow::Result;
use futures_util::Future;
use pyo3::prelude::*;

// use pyo3::types::*;

/// 获取 Python 的 None。
pub fn py_none() -> PyObject {
    Python::with_gil(|py| py.None())
}

/// 构造一个 Python 的 dict。
#[macro_export]
#[doc(hidden)]
macro_rules! py_dict {
    ($py:expr, $($name:expr => $value:expr),*) => {
        {
            let dict = pyo3::types::PyDict::new($py);
            $(
                dict.set_item($name, $value).expect("Failed to set_item on dict");
            )*
            dict
        }
    };
}

/// 等价于 `Some(py_dict!(..))`，用于指定 kwargs。
#[macro_export]
#[doc(hidden)]
macro_rules! kwargs {
    ($py:expr, $($name:expr => $value:expr),*) => {
        Some($crate::py_dict!($py, $($name => $value),*))
    };
}

/// 创建 Python 字符串（有缓存）。
#[macro_export]
macro_rules! py_intern {
    ($s:expr) => {
        Python::with_gil(|py| ::pyo3::types::PyString::intern(py, $s).into_py(py))
    };
}

/// 创建 Python 字符串（无缓存）。
#[macro_export]
macro_rules! py_str {
    ($s:expr) => {
        Python::with_gil(|py| ::pyo3::types::PyString::new(py, $s).into_py(py))
    };
}

#[macro_export]
macro_rules! import_call {
    ($py: expr, $module: expr => $attr: expr => $arg: expr) => {
        $py.import(::pyo3::intern!($py, $module))?
            .getattr(::pyo3::intern!($py, $attr))?
            .call1(($arg,))
    };
    ($py: expr, $module: expr => $attr: expr => @tuple $arg: expr) => {
        $py.import(::pyo3::intern!($py, $module))?
            .getattr(::pyo3::intern!($py, $attr))?
            .call1($arg)
    };
}

#[macro_export]
macro_rules! props {
    ($self_t: ident @ $cls: ident : $($name: ident => [$type: ty] $res: stmt);* ;) => {
        #[pymethods]
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
    F: Future<Output = Result<T, anyhow::Error>> + Send + 'static,
    T: IntoPy<PyObject>,
{
    pyo3_asyncio::tokio::future_into_py(py, async move { Ok(future.await?) })
}

/// 自动重试直到得到 `Ok(..)`。
pub async fn retry<F, T, D>(
    mut max_count: usize,
    mut f: impl FnMut() -> F,
    mut on_retry: impl FnMut(anyhow::Error, usize) -> D,
) -> Result<T>
where
    F: Future<Output = Result<T>>,
    D: Future<Output = ()>,
{
    loop {
        match f().await {
            Ok(t) => return Ok(t),
            Err(e) => {
                if max_count == 0 {
                    return Err(e);
                }
                max_count -= 1;
                on_retry(e, max_count).await;
            }
        }
    }
}

#[macro_export]
macro_rules! static_py_fn {
    ($name: ident, $cell_name: ident, $module: expr, [$($attr: expr),*]) => {
        #[allow(non_upper_case_globals)]
        static $cell_name: ::pyo3::once_cell::GILOnceCell<PyObject> = ::pyo3::once_cell::GILOnceCell::new();

        pub fn $name <'py>(python: pyo3::marker::Python<'py>) -> &'py pyo3::PyAny {
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
    ($pth: expr, $py: expr, ($($arg: expr),*)) => {
        $pth($py).call1(
            ($($arg),*)
        )
    };
    ($pth: expr, $py: expr, ($($arg: expr),*) ! $reason: expr) => {
        $pth($py).call1(
            ($($arg,)*)
        )
        .expect($reason)
        .into()
    }
}

static_py_fn!(
    datetime_from_ts,
    __DT_CELL,
    "datetime",
    ["datetime", "fromtimestamp"]
);
