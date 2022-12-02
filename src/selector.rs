//! // TODO

use anyhow::anyhow;
use pyo3::prelude::*;

use crate::client::Client;

/// 选择器。
#[pyclass(subclass)]
pub struct Selector {}

#[pymethods]
impl Selector {
    /// 执行选择器查询。
    ///
    /// # Python
    /// ```python
    /// async def do_query(self, client: Client) -> Any: ...
    /// ```
    pub fn do_query(&self, client: &Client) -> PyResult<PyObject> {
        let _ = client;
        Err(anyhow!("未实现"))?
    }
}

/// 好友选择器。
#[pyclass(extends=Selector)]
pub struct FriendSelector {
    pub(crate) uin: i64,
}

#[pymethods]
impl FriendSelector {
    /// 新建一个好友选择器。
    #[new]
    pub fn new(uin: i64) -> PyClassInitializer<FriendSelector> {
        PyClassInitializer::from(Selector {}).add_subclass(FriendSelector { uin })
    }
}

/// 多好友选择器。
#[pyclass(extends=Selector)]
pub struct MultiFriendSelector {
    pub(crate) uins: Vec<i64>,
}

#[pymethods]
impl MultiFriendSelector {
    /// 新建一个多好友选择器。
    #[new]
    pub fn new(uins: Vec<i64>) -> PyClassInitializer<MultiFriendSelector> {
        PyClassInitializer::from(Selector {}).add_subclass(MultiFriendSelector { uins })
    }
}

pub(crate) enum Selectors {}
