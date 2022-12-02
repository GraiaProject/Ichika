//! 好友分组。
//!
//! 更多信息参考 [`FriendGroup`]。

use std::sync::Arc;

use pyo3::prelude::*;
use ricq::structs::FriendGroupInfo;

use super::ClientImpl;

/// 好友分组。
///
/// # Python
/// ```python
/// class FriendGroup: ...
/// ```
#[pyclass]
#[derive(Clone)]
pub struct FriendGroup {
    #[allow(unused)] // TODO: remove this
    pub(super) client: Arc<ClientImpl>,
    pub(super) info: FriendGroupInfo,
}

#[pymethods]
impl FriendGroup {
    /// 好友分组 ID。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def id(self) -> int: ...
    /// ```
    #[getter]
    pub fn id(&self) -> u8 {
        self.info.group_id
    }

    /// 好友分组名称。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def name(self) -> str: ...
    /// ```
    #[getter]
    pub fn name(&self) -> &str {
        &self.info.group_name
    }

    /// 好友分组中的好友数量。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def count(self) -> int: ...
    /// ```
    #[getter]
    pub fn friend_count(&self) -> i32 {
        self.info.friend_count
    }

    /// 分组中在线的好友数量。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def online_count(self) -> int: ...
    /// ```
    #[getter]
    pub fn online_count(&self) -> i32 {
        self.info.online_friend_count
    }

    /// TODO: 未知
    ///
    /// # Python
    /// ```python
    /// @property
    /// def seq_id(self) -> int: ...
    /// ```
    #[getter]
    pub fn seq_id(&self) -> u8 {
        self.info.seq_id
    }

    fn __repr__(&self) -> String {
        format!(
            "FriendGroupInfo(id={:?}, name={:?}, friend_count={:?}, online_count={:?}, seq_id={:?})",
            self.id(),
            self.name(),
            self.friend_count(),
            self.online_count(),
            self.seq_id()
        )
    }
}
