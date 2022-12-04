use crate::py_dict;
use pyo3::{prelude::*, types::PyList};
use ricq::msg::{
    elem::{MarketFace, RQElem},
    MessageChain,
};

#[pyclass(module = "ichika.events.convert_msg#rs.inner")]
struct MarketFaceImpl {
    face: MarketFace,
}

#[pymethods]
impl MarketFaceImpl {
    #[getter]
    fn name(&self) -> String {
        self.face.name.clone()
    }
}

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
