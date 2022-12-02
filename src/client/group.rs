//! 群。

use std::sync::Arc;

use pyo3::prelude::*;
use ricq::structs::GroupInfo;

use super::ClientImpl;

/// 群聊。
#[pyclass]
pub struct Group {
    #[allow(unused)] // TODO: remove this
    pub(super) client: Arc<ClientImpl>,
    pub(super) info: GroupInfo,
}

#[pymethods]
impl Group {
    /// uin。
    ///
    /// 含义可参考：[#181](https://github.com/Mrs4s/MiraiGo/issues/181)。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def uin(self) -> int: ...
    /// ```
    #[getter]
    pub fn uin(&self) -> i64 {
        self.info.uin
    }

    /// 群号。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def code(self) -> int: ...
    /// ```
    #[getter]
    pub fn code(&self) -> i64 {
        self.info.code
    }

    /// 群名称。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def name(self) -> str: ...
    /// ```
    #[getter]
    pub fn name(&self) -> &str {
        &self.info.name
    }

    /// 入群公告。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def memo(self) -> str: ...
    /// ```
    #[getter]
    pub fn memo(&self) -> &str {
        &self.info.memo
    }

    /// 群主 QQ 号。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def owner_uin(self) -> int: ...
    /// ```
    #[getter]
    pub fn owner_uin(&self) -> i64 {
        self.info.owner_uin
    }

    /// 群创建时间。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def group_create_time(self) -> int: ...
    /// ```
    #[getter]
    pub fn group_create_time(&self) -> u32 {
        self.info.group_create_time
    }

    /// 群等级。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def level(self) -> int: ...
    /// ```
    #[getter]
    pub fn level(&self) -> u32 {
        self.info.group_level
    }

    /// 群成员数量。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def member_count(self) -> int: ...
    /// ```
    #[getter]
    pub fn member_count(&self) -> u16 {
        self.info.member_count
    }

    /// 最大群成员数量。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def max_member_count(self) -> int: ...
    /// ```
    #[getter]
    pub fn max_member_count(&self) -> u16 {
        self.info.max_member_count
    }

    /// 是否开启全员禁言。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def mute_all(self) -> bool: ...
    /// ```
    #[getter]
    pub fn mute_all(&self) -> bool {
        self.info.shut_up_timestamp != 0
    }

    /// 被禁言剩余时间，单位秒。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def my_shut_up_timestamp(self) -> int: ...
    /// ```
    #[getter]
    pub fn my_shut_up_timestamp(&self) -> i64 {
        self.info.my_shut_up_timestamp
    }

    /// 最后一条消息的 seq。
    ///
    /// 只有通过 [`get_group`] 或 [`get_groups`] 获取的群才有此字段。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def last_msg_seq(self) -> int: ...
    /// ```
    ///
    /// [`get_group`]: crate::client::Client::get_group
    /// [`get_groups`]: crate::client::Client::get_groups
    #[getter]
    pub fn last_msg_seq(&self) -> i64 {
        self.info.last_msg_seq
    }

    fn __repr__(&self) -> String {
        format!(
            "Group(uin={:?}, code={:?}, name={:?}, memo={:?}, owner_uin={:?}, group_create_time={:?}, \
                group_level={:?}, member_count={:?}, max_member_count={:?}, mute_all={:?}, \
                my_shut_up_timestamp={:?}, last_msg_seq={:?})",
            self.uin(),
            self.code(),
            self.name(),
            self.memo(),
            self.owner_uin(),
            self.group_create_time(),
            self.level(),
            self.member_count(),
            self.max_member_count(),
            self.mute_all(),
            self.my_shut_up_timestamp(),
            self.last_msg_seq()
        )
    }
}
