use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::*;
use ricq::msg::elem::{FlashImage, RQElem};
use ricq::msg::MessageChain;
use ricq_core::msg::elem::{At, Dice, Face, FingerGuessing, Text};

use super::elements::*;
use crate::{py_dict, static_py_fn};

pub fn serialize_audio(
    py: Python,
    url: String,
    ptt: &ricq_core::pb::msg::Ptt,
) -> PyResult<PyObject> {
    let audio_data = py_dict!(py,
        "type" => "Audio",
        "url" => url,
        "raw" => (SealedAudio {inner: ptt.clone()}).into_py(py)
    );
    let py_fn: &PyAny = py_deserialize(py);
    Ok(py_fn.call1((vec![audio_data],))?.into_py(py))
}

pub fn serialize_element(py: Python, e: RQElem) -> Option<&PyDict> {
    let data = match e {
        RQElem::At(a) => match a.target {
            0 => {
                py_dict!(py,
                    "type" => "AtAll"
                )
            }
            target => {
                py_dict!(py,
                    "type" => "At",
                    "target" => target,
                    "display" => a.display
                )
            }
        },
        RQElem::Text(t) => {
            py_dict!(py,
                "type" => "Text",
                "text" => t.content
            )
        }
        RQElem::Dice(d) => {
            py_dict!(py,
                "type" => "Dice",
                "value" => d.value
            )
        }
        RQElem::FingerGuessing(f) => {
            let choice = match f {
                FingerGuessing::Rock => "Rock",
                FingerGuessing::Paper => "Paper",
                FingerGuessing::Scissors => "Scissors",
            };
            py_dict!(py,
                "type" => "FingerGuessing",
                "choice" => choice
            )
        }
        RQElem::Face(f) => {
            py_dict!(py,
            "type" => "Face",
            "index" => f.index,
            "name" => f.name
            )
        }
        RQElem::MarketFace(m) => {
            let f = SealedMarketFace { inner: m };
            py_dict!(py,
            "type" => "MarketFace",
            "raw" => f.into_py(py)
            )
        }
        RQElem::GroupImage(i) => {
            py_dict!(py,
            "type" => "Image",
            "url" => i.url(),
            "raw" => (SealedGroupImage {inner: i}).into_py(py)
            )
        }
        RQElem::FriendImage(i) => {
            py_dict!(py,
            "type" => "Image",
            "url" => i.url(),
            "raw" => (SealedFriendImage {inner: i}).into_py(py)
            )
        }
        RQElem::FlashImage(i) => match i {
            FlashImage::GroupImage(i) => {
                py_dict!(py,
                "type" => "FlashImage",
                "url" => i.url(),
                "raw" => (SealedGroupImage {inner: i}).into_py(py)
                )
            }
            FlashImage::FriendImage(i) => {
                py_dict!(py,
                "type" => "FlashImage",
                "url" => i.url(),
                "raw" => (SealedFriendImage {inner: i}).into_py(py)
                )
            }
        },
        RQElem::Other(_) => {
            return None;
        }
        unhandled => {
            py_dict!(py,
                "type" => "Unknown",
                "raw" => format!("{unhandled:?}")
            )
        }
    };
    Some(data)
}

pub fn serialize_message_chain(py: Python, chain: MessageChain) -> PyResult<Py<PyList>> {
    let res = PyList::empty(py);
    for e in chain {
        if let Some(data) = serialize_element(py, e) {
            res.append(data)?;
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

pub fn deserialize_element(chain: &mut MessageChain, ident: &str, store: &PyDict) -> PyResult<()> {
    match ident {
        "AtAll" => chain.push(At::new(0)),
        "At" => {
            if let Some(t) = store.get_item("target") {
                chain.push(At::new(t.extract::<i64>()?));
            }
        }
        "Text" => {
            if let Some(t) = store.get_item("text") {
                chain.push(Text::new(t.extract::<String>()?));
            }
        }
        "Dice" => {
            if let Some(t) = store.get_item("value") {
                chain.push(Dice::new(t.extract::<i32>()?));
            }
        }
        "FingerGuessing" => {
            if let Some(t) = store.get_item("choice") {
                chain.push(match t.extract::<&str>()? {
                    "Rock" => FingerGuessing::Rock,
                    "Paper" => FingerGuessing::Paper,
                    "Scissors" => FingerGuessing::Scissors,
                    _ => return Ok(()),
                });
            }
        }
        "MarketFace" => {
            if let Some(t) = store.get_item("raw") {
                chain.push(t.extract::<SealedMarketFace>()?.inner);
            }
        }
        "Face" => {
            if let Some(t) = store.get_item("index") {
                chain.push(Face::new(t.extract::<i32>()?));
            }
        }
        "Image" => {
            if let Some(t) = store.get_item("raw") {
                match t.extract::<SealedFriendImage>() {
                    Ok(i) => chain.push(i.inner),
                    Err(_) => chain.push(t.extract::<SealedGroupImage>()?.inner),
                };
            }
        }
        "FlashImage" => {
            if let Some(t) = store.get_item("raw") {
                match t.extract::<SealedFriendImage>() {
                    Ok(i) => chain.push(FlashImage::from(i.inner)),
                    Err(_) => chain.push(FlashImage::from(t.extract::<SealedGroupImage>()?.inner)),
                };
            }
        }
        _ => {}
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
        deserialize_element(&mut chain, name, elem_d)?;
    }
    Ok(chain)
}
