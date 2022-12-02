//! 账号信息。
//!
//! 更多信息参考 [`AccountInfo`]。

use pyo3::{prelude::*, types::*};

/// 账号信息。
///
/// # Python
/// ```python
/// class AccountInfo:
///     @property
///     def nickname(self) -> str: ...
///     @property
///     def age(self) -> int: ...
///     @property
///     def gender(self) -> int: ...
/// ```
#[pyclass]
#[derive(Debug, Clone)]
pub struct AccountInfo {
    /// 昵称。
    #[pyo3(get)]
    pub nickname: Py<PyString>,

    /// 年龄。
    #[pyo3(get)]
    pub age: u8,

    /// 性别。
    #[pyo3(get)]
    pub gender: u8,
}

#[pymethods]
impl AccountInfo {
    fn __repr__(&self) -> String {
        Python::with_gil(|py| {
            format!(
                "AccountInfo(nickname={:?}, age={:?}, gender={:?})",
                self.nickname.as_ref(py).repr().unwrap(),
                self.age,
                self.gender
            )
        })
    }
}
