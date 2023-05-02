use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::*;
use pyo3_repr::PyRepr;
use ricq::structs::{ForwardMessage, MessageReceipt, MusicShare, MusicVersion};
use ricq_core::command::oidb_svc::OcrResponse;

use crate::utils::{datetime_from_ts, py_try, py_use};
#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct AccountInfo {
    pub nickname: String,
    pub age: u8,
    pub gender: u8,
}

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct OtherClientInfo {
    pub app_id: i64,
    pub instance_id: i32,
    pub sub_platform: String,
    pub device_kind: String,
}

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct RawMessageReceipt {
    pub seq: i32,
    pub rand: i32,
    pub raw_seqs: Py<PyTuple>,
    pub raw_rands: Py<PyTuple>,
    pub time: PyObject, // datetime
    pub kind: String,
    pub target: i64,
}

impl RawMessageReceipt {
    pub fn new(origin: MessageReceipt, kind: impl Into<String>, target: i64) -> PyResult<Self> {
        let kind: String = kind.into();
        let MessageReceipt { seqs, rands, time } = origin;
        let seq: i32 = *seqs
            .first()
            .ok_or_else(|| PyIndexError::new_err("Empty returning seqs"))?;
        let rand: i32 = *rands
            .first()
            .ok_or_else(|| PyIndexError::new_err("Empty returning rands"))?;
        py_try(|py| {
            let time = datetime_from_ts(py, time)?.to_object(py);
            Ok(Self {
                seq,
                rand,
                raw_seqs: PyTuple::new(py, seqs).into_py(py),
                raw_rands: PyTuple::new(py, rands).into_py(py),
                time,
                kind,
                target,
            })
        })
    }

    pub fn empty(kind: impl Into<String>, target: i64) -> PyResult<Self> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map_err(|_| PyValueError::new_err("SystemTime before UNIX EPOCH"))?;
        Self::new(
            MessageReceipt {
                seqs: vec![0],
                rands: vec![0],
                time: timestamp.as_secs() as i64,
            },
            kind,
            target,
        )
    }
}

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct OCRResult {
    pub texts: Py<PyTuple>, // PyTuple<OCRText>
    pub language: String,
}

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct OCRText {
    pub detected_text: String,
    pub confidence: i32,
    pub polygon: Option<Py<PyTuple>>, // PyTuple<(i32, i32))>
    pub advanced_info: String,
}

impl From<OcrResponse> for OCRResult {
    fn from(value: OcrResponse) -> Self {
        py_use(|py| {
            let OcrResponse { texts, language } = value;
            let text_iter = texts.into_iter().map(|txt| {
                let polygon = txt.polygon.map(|poly| {
                    PyTuple::new(
                        py,
                        poly.coordinates
                            .into_iter()
                            .map(|coord| (coord.x, coord.y).to_object(py)),
                    )
                    .into_py(py)
                });
                OCRText {
                    detected_text: txt.detected_text,
                    confidence: txt.confidence,
                    polygon,
                    advanced_info: txt.advanced_info,
                }
                .into_py(py)
            });
            OCRResult {
                texts: PyTuple::new(py, text_iter).into_py(py),
                language,
            }
        })
    }
}

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
