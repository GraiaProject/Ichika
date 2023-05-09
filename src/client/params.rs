use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::*;
use ricq::structs::{ForwardMessage, MusicShare, MusicVersion};

use crate::utils::py_try;

#[derive(FromPyObject)]
pub enum OnlineStatusParam {
    #[pyo3(annotation = "tuple[bool, int]")]
    Normal(bool, i32),
    #[pyo3(annotation = "tuple[int, str]")]
    Custom(u64, String),
}

impl From<OnlineStatusParam> for ricq::structs::Status {
    fn from(value: OnlineStatusParam) -> Self {
        use ricq::structs::{CustomOnlineStatus, Status};
        match value {
            OnlineStatusParam::Custom(face_index, wording) => Status {
                online_status: 11,
                ext_online_status: 2000,
                custom_status: Some(CustomOnlineStatus {
                    face_index,
                    wording,
                }),
            },
            OnlineStatusParam::Normal(is_ext, index) => Status {
                online_status: if is_ext { 11 } else { index },
                ext_online_status: if is_ext { i64::from(index) } else { 0 },
                custom_status: None,
            },
        }
    }
}

#[derive(FromPyObject)]
pub struct MusicShareParam {
    #[pyo3(attribute)]
    kind: String,
    #[pyo3(attribute)]
    title: String,
    #[pyo3(attribute)]
    summary: String,
    #[pyo3(attribute)]
    jump_url: String,
    #[pyo3(attribute)]
    picture_url: String,
    #[pyo3(attribute)]
    music_url: String,
    #[pyo3(attribute)]
    brief: String,
}

impl TryFrom<MusicShareParam> for (MusicShare, MusicVersion) {
    type Error = PyErr;

    fn try_from(value: MusicShareParam) -> Result<Self, Self::Error> {
        let MusicShareParam {
            kind,
            title,
            summary,
            jump_url,
            picture_url,
            music_url,
            brief,
        } = value;
        let version = match kind.as_str() {
            "QQ" => MusicVersion::QQ,
            "Netease" => MusicVersion::NETEASE,
            "Migu" => MusicVersion::MIGU,
            "Kugou" => MusicVersion::KUGOU,
            "Kuwo" => MusicVersion::KUWO,
            platform => {
                return Err(PyValueError::new_err(format!(
                    "无法识别的音乐平台: {platform}"
                )))
            }
        };
        let share = MusicShare {
            title,
            brief,
            summary,
            url: jump_url,
            picture_url,
            music_url,
        };
        Ok((share, version))
    }
}

pub struct PyForwardMessage {
    sender_id: i64,
    time: i32,
    sender_name: String,
    content: PyInnerForward,
}


impl TryFrom<PyForwardMessage> for ForwardMessage {
    type Error = PyErr;

    fn try_from(value: PyForwardMessage) -> PyResult<Self> {
        use ricq::structs::{ForwardNode, MessageNode};

        use crate::message::convert::deserialize_message_chain;

        let PyForwardMessage {
            sender_id,
            time,
            sender_name,
            content,
        } = value;
        Ok(match content {
            PyInnerForward::Message(msg) => Self::Message(MessageNode {
                sender_id,
                time,
                sender_name,
                elements: py_try(|py| deserialize_message_chain(msg.as_ref(py)))?,
            }),
            PyInnerForward::Forward(fwd) => Self::Forward(ForwardNode {
                sender_id,
                time,
                sender_name,
                nodes: fwd.into_iter().map(|v| v.try_into()).try_collect()?,
            }),
        })
    }
}

pub enum PyInnerForward {
    Forward(Vec<PyForwardMessage>),
    Message(Py<PyList>),
}

impl<'s> FromPyObject<'s> for PyForwardMessage {
    fn extract(obj: &'s PyAny) -> PyResult<Self> {
        let typ: String = obj.get_item("type")?.extract()?;
        let content: &PyList = obj.get_item("content")?.extract()?;
        Ok(Self {
            sender_id: obj.get_item("sender_id")?.extract()?,
            time: obj.get_item("time")?.extract()?,
            sender_name: obj.get_item("sender_name")?.extract()?,
            content: match typ.as_str() {
                "Forward" => {
                    PyInnerForward::Forward(content.into_iter().map(|o| o.extract()).try_collect()?)
                }
                "Message" => PyInnerForward::Message(content.into_py(content.py())),
                _ => Err(PyTypeError::new_err("Invalid forward content type"))?,
            },
        })
    }
}
