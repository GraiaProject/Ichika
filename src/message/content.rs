//! 消息内容。
//!
//! - `MessageSegments`: `list[str | Element]` 形式的分段消息，终端构造用。
//! - `MessageContent`: 封装后的消息内容，用于发送和处理。
//! - `MessageChain`: `ricq` 的消息链，对用户不可见。

use anyhow::Result;
use pyo3::prelude::*;
use ricq::msg::PushElem;
use ricq_core::msg::MessageChain;

use super::elements::ElementOrText;
use crate::utils::py_future;

/// 消息内容。
///
/// # Examples
/// ```python
/// from awr import MessageContent, Face
///
/// message = await MessageContent.build_friend_message("Hello, world!", Face(1))
/// client.friend(123456789).send(message)
/// ```
///
/// # Python
/// ```python
/// class MessageContent: ...
/// ```
#[pyclass]
#[derive(Clone)]
pub struct MessageContent {
    chain: MessageChain,
}

impl MessageContent {
    fn new(chain: MessageChain) -> Self {
        Self { chain }
    }
}

impl MessageContent {
    pub(crate) async fn build_friend_message_impl(
        segments: Vec<ElementOrText>,
    ) -> Result<MessageContent> {
        let mut elems = vec![];
        for elem in segments.into_iter() {
            PushElem::push_to(elem, &mut elems);
        }

        let chain = MessageChain::new(elems);
        let content = MessageContent::new(chain);
        Ok(content)
    }
}

#[pymethods]
impl MessageContent {
    /// 构造好友消息链。
    ///
    /// # Python
    /// ```python
    /// @staticmethod
    /// async def build_friend_message(*segments: str | Element) -> MessageContent: ...
    /// ```
    #[staticmethod]
    #[args(segments = "*")]
    pub fn build_friend_message(py: Python, segments: Vec<ElementOrText>) -> PyResult<&PyAny> {
        py_future(py, MessageContent::build_friend_message_impl(segments))
    }
}

impl From<MessageContent> for MessageChain {
    fn from(content: MessageContent) -> Self {
        content.chain
    }
}
