use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::*;
use ricq::msg::elem::{FlashImage, RQElem, Reply};
use ricq::msg::MessageChain;
use ricq_core::msg::elem::{At, Dice, Face, FingerGuessing, Text};

use super::elements::*;
use crate::utils::datetime_from_ts;
use crate::{dict, static_py_fn};

pub fn serialize_audio_dict<'py>(
    py: Python<'py>,
    url: String,
    ptt: &ricq_core::pb::msg::Ptt,
) -> PyResult<&'py PyDict> {
    Ok(dict! {py,
        type: "Audio",
        url: url,
        raw: SealedAudio {inner: ptt.clone()}.into_py(py),
    })
}
pub fn serialize_audio(
    py: Python,
    url: String,
    ptt: &ricq_core::pb::msg::Ptt,
) -> PyResult<PyObject> {
    let audio_data = serialize_audio_dict(py, url, ptt)?;
    let py_fn: &PyAny = py_deserialize(py);
    Ok(py_fn.call1((vec![audio_data],))?.into_py(py))
}

pub fn serialize_element(py: Python, e: RQElem) -> PyResult<Option<&PyDict>> {
    let data = match e {
        RQElem::At(a) => match a.target {
            0 => {
                dict! {py, type: "AtAll"}
            }
            target => {
                dict! {py,
                    type: "At",
                    target: target,
                    display: a.display,
                }
            }
        },
        RQElem::Text(t) => {
            dict! {py,
                type: "Text",
                text: t.content,
            }
        }
        RQElem::Dice(d) => {
            dict! {py,
                type: "Dice",
                value: d.value,
            }
        }
        RQElem::FingerGuessing(f) => {
            let choice = match f {
                FingerGuessing::Rock => "Rock",
                FingerGuessing::Paper => "Paper",
                FingerGuessing::Scissors => "Scissors",
            };
            dict! {py,
                type: "FingerGuessing",
                choice: choice
            }
        }
        RQElem::Face(f) => {
            dict! {py,
                type: "Face",
                index: f.index,
                name: f.name
            }
        }
        RQElem::MarketFace(m) => {
            let f = SealedMarketFace { inner: m };
            dict! {py,
            type: "MarketFace",
            raw: f.into_py(py)
            }
        }
        RQElem::GroupImage(i) => {
            dict! {py,
            type: "Image",
            url: i.url(),
            raw: (SealedGroupImage {inner: i}).into_py(py)
            }
        }
        RQElem::FriendImage(i) => {
            dict! {py,
            type: "Image",
            url: i.url(),
            raw: (SealedFriendImage {inner: i}).into_py(py)
            }
        }
        RQElem::FlashImage(i) => match i {
            FlashImage::GroupImage(i) => {
                dict! {py,
                type: "FlashImage",
                url: i.url(),
                raw: (SealedGroupImage {inner: i}).into_py(py)
                }
            }
            FlashImage::FriendImage(i) => {
                dict! {py,
                type: "FlashImage",
                url: i.url(),
                raw: (SealedFriendImage {inner: i}).into_py(py)
                }
            }
        },
        RQElem::Other(_) => {
            return Ok(None);
        }
        unhandled => {
            dict! {py,
                type: "Unknown",
                raw: format!("{unhandled:?}")
            }
        }
    };
    Ok(Some(data))
}

// Reply + Bot Image = skip message ???
// Needs testing
pub fn serialize_reply(py: Python, reply: Reply) -> PyResult<&PyDict> {
    Ok(dict! {py,
        type: "Reply",
        seq: reply.reply_seq,
        sender: reply.sender,
        time: datetime_from_ts(py, reply.time)?,
        content: reply.elements.to_string()
    })
}

pub fn serialize_message_chain(py: Python, chain: MessageChain) -> PyResult<Py<PyList>> {
    use ricq_core::msg::MessageElem as BaseElem;
    let res = PyList::empty(py);
    for e in chain.0 {
        match e {
            BaseElem::SrcMsg(reply) => {
                res.append(serialize_reply(py, reply.into())?)?;
            }
            BaseElem::AnonGroupMsg(_) => {} // Anonymous information, TODO
            elem => {
                if let Some(data) = serialize_element(py, RQElem::from(elem))? {
                    res.append(data)?;
                }
            }
        }
    }
    Ok(res.into_py(py))
}

static_py_fn!(
    py_deserialize,
    __py_deserialize_cell,
    "ichika.message",
    ["deserialize_message"]
);

pub fn serialize_as_py_chain(py: Python, chain: MessageChain) -> PyResult<PyObject> // PyMessageChain
{
    let py_fn: &PyAny = py_deserialize(py);
    Ok(py_fn
        .call1((serialize_message_chain(py, chain)?,))?
        .into_py(py))
}

pub fn deserialize_element(chain: &mut MessageChain, ident: &str, store: &PyAny) -> PyResult<()> {
    match ident {
        "AtAll" => chain.push(At::new(0)),
        "At" => {
            chain.push(At::new(store.get_item("target")?.extract::<i64>()?));
        }
        "Text" => {
            chain.push(Text::new(store.get_item("text")?.extract::<String>()?));
        }
        "Dice" => {
            chain.push(Dice::new(store.get_item("value")?.extract::<i32>()?));
        }
        "FingerGuessing" => {
            chain.push(match store.get_item("choice")?.extract::<&str>()? {
                "Rock" => FingerGuessing::Rock,
                "Paper" => FingerGuessing::Paper,
                "Scissors" => FingerGuessing::Scissors,
                _ => return Ok(()),
            });
        }
        "MarketFace" => {
            chain.push(store.get_item("raw")?.extract::<SealedMarketFace>()?.inner);
        }
        "Face" => {
            chain.push(Face::new(store.get_item("index")?.extract::<i32>()?));
        }
        "Image" => {
            let raw = store.get_item("raw")?;
            match raw.extract::<SealedFriendImage>() {
                Ok(i) => chain.push(i.inner),
                Err(_) => chain.push(raw.extract::<SealedGroupImage>()?.inner),
            };
        }
        "FlashImage" => {
            let raw = store.get_item("raw")?;
            match raw.extract::<SealedFriendImage>() {
                Ok(i) => chain.push(FlashImage::from(i.inner)),
                Err(_) => chain.push(FlashImage::from(raw.extract::<SealedGroupImage>()?.inner)),
            };
        }
        "Reply" => {
            let seq: i32 = store.get_item("seq")?.extract()?;
            let sender: i64 = store.get_item("sender")?.extract()?;
            let time: i32 = store.get_item("time")?.extract()?;
            let content: String = store.get_item("content")?.extract()?;
            chain.push(vec![Reply {
                reply_seq: seq,
                sender,
                time,
                elements: MessageChain::new(Text::new(content)),
            }
            .into()]);
        }
        _ => {
            return Err(PyTypeError::new_err(format!(
                "无法处理元素 {ident} {store}"
            )))
        }
    }
    Ok(())
}

pub fn deserialize_message_chain(list: &PyList) -> PyResult<MessageChain> {
    let mut chain: MessageChain = MessageChain::new(Vec::new());
    for elem_d in list {
        let elem_d: &PyDict = elem_d.downcast()?;
        let name = elem_d
            .get_item("type")
            .ok_or_else(|| PyValueError::new_err("Missing `type`!"))?
            .extract::<&str>()?;
        deserialize_element(&mut chain, name, elem_d.into())?;
    }
    tracing::info!("{:?}", chain);
    Ok(chain)
}
