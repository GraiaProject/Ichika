use super::elements::MarketFaceImpl;
use crate::py_dict;
use pyo3::{prelude::*, types::*};
use ricq::msg::{elem::RQElem, MessageChain};
use ricq_core::msg::elem::{At, Dice, Face, FingerGuessing, Text};
pub fn convert_message_chain(py: Python, chain: MessageChain) -> PyResult<Py<PyList>> {
    let res = PyList::empty(py);
    for e in chain {
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
                    ricq::msg::elem::FingerGuessing::Rock => "Rock",
                    ricq::msg::elem::FingerGuessing::Paper => "Paper",
                    ricq::msg::elem::FingerGuessing::Scissors => "Scissors",
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
                let f = MarketFaceImpl { face: m };
                py_dict!(py,
                "type" => "MarketFace",
                "raw" => f.into_py(py)
                )
            }
            RQElem::Other(_) => {
                continue;
            }
            unhandled => {
                py_dict!(py,
                    "type" => "Unknown",
                    "raw" => format!("{:?}", unhandled)
                )
            }
        };
        res.append(data)?
    }
    Ok(res.into_py(py))
}

pub fn extract_message_chain(py: Python, list: Py<PyList>) -> PyResult<MessageChain> {
    let mut chain: MessageChain = MessageChain::new(Vec::new());
    for elem_d in list.as_ref(py) {
        let elem_d: &PyDict = elem_d.downcast()?;
        if let Some(key) = elem_d.get_item("type") {
            match key.extract::<&str>()? {
                "AtAll" => chain.push(At::new(0)),
                "At" => {
                    if let Some(t) = elem_d.get_item("target") {
                        chain.push(At::new(t.extract::<i64>()?))
                    }
                }
                "Text" => {
                    if let Some(t) = elem_d.get_item("text") {
                        chain.push(Text::new(t.extract::<String>()?))
                    }
                }
                "Dice" => {
                    if let Some(t) = elem_d.get_item("value") {
                        chain.push(Dice::new(t.extract::<i32>()?))
                    }
                }
                "FingerGuessing" => {
                    if let Some(t) = elem_d.get_item("choice") {
                        chain.push(match t.extract::<&str>()? {
                            "Rock" => FingerGuessing::Rock,
                            "Paper" => FingerGuessing::Paper,
                            "Scissors" => FingerGuessing::Scissors,
                            _ => continue,
                        })
                    }
                }
                "MarketFace" => {
                    if let Some(t) = elem_d.get_item("raw") {
                        chain.push(t.extract::<MarketFaceImpl>()?.face)
                    }
                }
                "Face" => {
                    if let Some(t) = elem_d.get_item("index") {
                        chain.push(Face::new(t.extract::<i32>()?))
                    }
                }
                _ => continue,
            }
        }
    }
    Ok(chain)
}

#[pyfunction]
pub fn preview_raw_chain(py: Python, list: Py<PyList>) -> PyResult<String> {
    Ok(format!("{:?}", extract_message_chain(py, list)?))
}
