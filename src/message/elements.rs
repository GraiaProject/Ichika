//! 消息元素。

use pyo3::{exceptions::PyTypeError, prelude::*, types::*};
use ricq::msg::PushElem;
use ricq_core::msg::elem;

/// 消息元素。
///
/// # Python
/// ```python
/// Element = At | Face
/// ```
#[derive(FromPyObject)]
#[non_exhaustive]
pub enum Element {
    #[doc(hidden)]
    At(At),
    #[doc(hidden)]
    Face(Face),
    #[doc(hidden)]
    Dice(Dice),
}

impl PushElem for Element {
    fn push_to(elem: Self, vec: &mut Vec<ricq::msg::MessageElem>) {
        match elem {
            Element::At(at) => PushElem::push_to(elem::At::from(at), vec),
            Element::Face(face) => PushElem::push_to(elem::Face::from(face), vec),
            Element::Dice(dice) => PushElem::push_to(elem::Dice::from(dice), vec)
        }
    }
}

#[doc(hidden)]
#[derive(FromPyObject)]
pub enum ElementOrText {
    Element(Element),
    Text(String),
}

impl PushElem for ElementOrText {
    fn push_to(elem: Self, vec: &mut Vec<ricq::msg::MessageElem>) {
        match elem {
            ElementOrText::Element(elem) => PushElem::push_to(elem, vec),
            ElementOrText::Text(text) => PushElem::push_to(elem::Text::new(text), vec),
        }
    }
}

/// 消息元素基类。
#[pyclass(subclass)]
pub struct ElementBase {}

#[pymethods]
impl ElementBase {
    #[new]
    fn new() -> PyResult<Self> {
        Err(PyTypeError::new_err("ElementBase is abstract"))
    }
}

/// At。
///
/// # Python
/// ```python
/// class At():
///     @property
///     def target(self) -> int: ...
/// ```
#[pyclass]
#[derive(Clone)]
pub struct At {
    /// 被 At 的 QQ 号。
    #[pyo3(get)]
    pub target: i64,
}

#[pymethods]
impl At {
    /// 构造 At 消息元素。
    ///
    /// # Arguments
    /// * `target` - 被 At 的 QQ 号。
    #[new]
    pub fn new(target: i64) -> Self {
        Self { target }
    }
}

impl From<At> for elem::At {
    fn from(at: At) -> Self {
        Self::new(at.target)
    }
}

/// 表情。
///
/// # Python
/// ```python
/// class Face(TypedDict):
///     ...
/// ```
#[pyclass]
#[derive(Clone)]
pub struct Face {
    elem: elem::Face,
}

#[pymethods]
impl Face {
    /// 构造表情消息元素。
    ///
    /// # Python
    /// ```python
    /// @overload
    /// def __init__(self, id: int, /) -> None: ...
    /// @overload
    /// def __init__(self, name: str, /) -> None: ...
    /// @overload
    /// def __init__(self, *, id: int | None = None, name: str | None = None) -> None: ...
    /// ```
    #[new]
    #[args(args = "*", kwargs = "**")]
    pub fn new(args: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<Self> {
        match args.len() {
            0 => {
                let id = kwargs
                    .and_then(|kwargs| kwargs.get_item("id"))
                    .map(|id| id.extract())
                    .transpose()?;
                if let Some(id) = id {
                    return Ok(Self {
                        elem: elem::Face::new(id),
                    });
                }
                let name = kwargs
                    .and_then(|kwargs| kwargs.get_item("name"))
                    .map(|name| name.extract())
                    .transpose()?;
                if let Some(name) = name {
                    let elem = elem::Face::new_from_name(name)
                        .ok_or_else(|| PyTypeError::new_err("invalid face name"))?;
                    return Ok(Self { elem });
                }
                Err(PyTypeError::new_err(
                    "missing required argument 'id' or 'name'",
                ))
            }
            1 => {
                let id_or_name = args.get_item(0)?;
                if id_or_name.is_instance_of::<PyString>()? {
                    let name = id_or_name.extract()?;
                    let elem = elem::Face::new_from_name(name)
                        .ok_or_else(|| PyTypeError::new_err("invalid face name"))?;
                    Ok(Self { elem })
                } else {
                    let id = id_or_name.extract()?;
                    Ok(Self {
                        elem: elem::Face::new(id),
                    })
                }
            }
            _ => Err(PyTypeError::new_err("expected at most 1 arguments")),
        }
    }

    /// 表情 id。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def id(self) -> int: ...
    /// ```
    #[getter]
    pub fn id(&self) -> i32 {
        self.elem.index
    }

    /// 表情名称。
    ///
    /// # Python
    /// ```python
    /// @property
    /// def name(self) -> str: ...
    /// ```
    #[getter]
    pub fn name(&self) -> String {
        self.elem.name.to_owned()
    }
}

impl From<Face> for elem::Face {
    fn from(face: Face) -> Self {
        face.elem
    }
}

///骰子。
#[pyclass]
#[derive(Clone)]
pub struct Dice {
    #[pyo3(get)]
    value: i32,
}

#[pymethods]
impl Dice {
    /// 构造新的骰子元素。
    /// 
    /// # Python
    /// ```python
    /// @overload
    /// def __init__(self, value: int) -> None: ...
    /// ```
    #[new]
    fn new(value: i32) -> PyResult<Self> {
        Ok(Self { value })
    }
}

impl From<Dice> for elem::Dice {
    fn from(value: Dice) -> Self {
        elem::Dice::new(value.value)
    }
}
