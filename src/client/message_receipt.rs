//! 消息回执。

use std::sync::Arc;

use pyo3::prelude::*;
use ricq::structs::MessageReceipt as Receipt;

use super::{client_impl::ClientImpl, friend::FriendSelector};

pub(crate) enum MesageReceiptContext {
    #[allow(unused)] // TODO: remove this
    Group {
        group_id: i64,
        target_id: i64,
    },
    Friend(FriendSelector),
}

/// 消息回执，可以用于撤回消息。
///
/// # Examples
/// ```python
/// receipt = await client.friend(123456789).send(['hello'])
/// await receipt.recall()
/// ```
///
/// # Python
/// ```python
/// class MessageReceipt: ...
/// ```
#[pyclass]
pub struct MessageReceipt {
    pub(crate) context: MesageReceiptContext,
    pub(crate) receipt: Receipt,
}

impl MessageReceipt {
    pub(super) fn new_from_friend(client: Arc<ClientImpl>, uin: i64, receipt: Receipt) -> Self {
        Self {
            context: MesageReceiptContext::Friend(FriendSelector { client, uin }),
            receipt,
        }
    }
}

#[pymethods]
impl MessageReceipt {
    /// 消息发送时间。
    pub fn msg_time(&self) -> i64 {
        self.receipt.time
    }

    /// 消息 seqs。
    pub fn seqs(&self) -> Vec<i32> {
        self.receipt.seqs.clone()
    }

    /// 消息 rands。
    pub fn rands(&self) -> Vec<i32> {
        self.receipt.rands.clone()
    }

    /// 撤回消息。
    ///
    /// # Python
    /// ```python
    /// async def recall(self) -> None: ...
    /// ```
    pub fn recall<'py>(self_: &'py PyCell<Self>, py: Python<'py>) -> PyResult<&'py PyAny> {
        match &self_.borrow().context {
            MesageReceiptContext::Group { .. } => todo!(),
            MesageReceiptContext::Friend(selector) => selector.recall(py, self_.borrow()),
        }
    }

    // TODO: 持久化？
}
