//! QQ 无头客户端。
//!
//! 更多信息参考 [`Client`]。

use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use pyo3::{prelude::*, types::*};
use tokio::task::JoinHandle;

mod client_impl;

pub mod account_info;
pub mod friend;
pub mod friend_group;
pub mod friend_list;
pub mod group;
pub mod message_receipt;

use crate::{
    login::reconnect,
    py_intern,
    utils::{py_future, py_none, py_obj},
};

use self::{
    account_info::AccountInfo, client_impl::ClientImpl, friend::FriendSelector,
    friend_list::FriendList, group::Group,
};

/// QQ 无头客户端。
///
/// # Examples
/// ```python
/// client = await Dynamic().login(12345678, "./bots")
///
/// friend_list = client.get_friend_list()
/// for friend in friend_list.friends():
///     print(friend.nickname())
///
/// await client.alive()
/// ```
///
/// # Python
/// ```python
/// class Client: ...
/// ```
#[pyclass]
pub struct Client {
    client: Arc<ClientImpl>,
    alive: Option<JoinHandle<()>>,
    uin: i64,
    data_folder: PathBuf,
}

impl Client {
    pub(crate) async fn new(
        client: Arc<ricq::Client>,
        alive: JoinHandle<()>,
        data_folder: PathBuf,
    ) -> Self {
        let uin = client.uin().await;
        let client = Arc::new(ClientImpl::new(client));
        Self {
            client,
            alive: Some(alive),
            uin,
            data_folder,
        }
    }
}

#[pymethods]
impl Client {
    /// 等待并保持客户端连接，期间会自动进行断线重连。
    ///
    /// 多次调用此方法时，后续的调用将直接返回。
    ///
    pub fn alive<'py>(&mut self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.inner().clone();
        let data_folder = self.data_folder.clone();
        let alive = self.alive.take();
        py_future(py, async move {
            if let Some(mut alive) = alive {
                loop {
                    alive.await?;

                    // 断线重连
                    if let Some(handle) = reconnect(&client, &data_folder).await? {
                        alive = handle;
                    } else {
                        break;
                    }
                }
            }
            tracing::info!("客户端 {} 连接断开", client.uin().await);
            Ok(py_none())
        })
    }

    /// 客户端 QQ 号。
    #[getter]
    pub fn uin(&self) -> i64 {
        self.uin
    }

    /// 是否在线。
    ///
    /// # Python
    /// ```python
    /// def is_online(self) -> bool: ...
    /// ```
    pub fn is_online(&self) -> bool {
        self.client
            .inner()
            .online
            .load(std::sync::atomic::Ordering::Acquire)
    }

    /// 构造好友选择器。
    ///
    /// 参考 [`FriendSelector`]。
    ///
    /// # Examples
    /// ```python
    /// await client.friend(12345678).poke()
    /// ```
    ///
    /// # Python
    /// ```python
    /// def friend(self, uin: int) -> FriendSelector: ...
    /// ```    
    pub fn friend(&self, uin: i64) -> FriendSelector {
        FriendSelector {
            client: self.client.clone(),
            uin,
        }
    }

    /// 获取账号信息。
    ///
    /// 参考 [`AccountInfo`]。
    ///
    /// # Examples
    /// ```python
    /// info = await client.get_account_info()
    /// print("nickname:", info.nickname)
    /// print("age:", info.age)
    /// print("gender:", info.gender)
    /// ```
    ///
    /// # Python
    /// ```python
    /// async def get_account_info(self) -> AccountInfo: ...
    /// ```
    pub fn get_account_info<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.inner().clone();
        py_future(py, async move {
            let info = client.account_info.read().await;
            let info = AccountInfo {
                nickname: py_intern!(&info.nickname),
                age: info.age,
                gender: info.gender,
            };
            Ok(py_obj(info)?)
        })
    }

    /// 获取好友列表。
    ///
    /// 参考 [`FriendList`]。
    ///
    /// # Examples
    /// ```python
    /// friend_list = await client.get_friend_list()
    /// for friend in friend_list.friends():
    ///     print(friend.nickname)
    /// ```
    ///
    /// # Python
    /// ```python
    /// async def friend_list(self) -> FriendList: ...
    /// ```
    pub fn get_friend_list<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let friend_list = client.get_friend_list_cached().await?;
            Ok(py_obj(friend_list)?)
        })
    }

    /// 获取遍历好友信息的迭代器。
    ///
    /// 参考 [`Friend`]。
    ///
    /// # Examples
    /// ```python    
    /// for friend in await client.get_friends():
    ///     print(friend.nickname)
    /// ```
    ///
    /// # Python
    /// ```python
    /// async def get_friends(self) -> Iterator[Friend]:
    /// ```
    ///
    /// [`Friend`]: friend::Friend
    pub fn get_friends<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let friend_list = client.get_friend_list_cached().await?;
            let friends = Python::with_gil(|py| -> PyResult<PyObject> {
                Ok(FriendList::friends(Py::new(py, friend_list)?, py).into_py(py))
            })?;
            Ok(friends)
        })
    }

    /// 查找指定的好友。
    ///
    /// 参考 [`Friend`]。
    ///
    /// # Examples
    /// ```python    
    /// friend = await client.get_friend(12345678)
    /// if friend:
    ///     print(friend.nickname)
    /// else:
    ///     print("未找到好友 12345678")
    /// ```
    ///
    /// # Python
    /// ```python
    /// async def get_friend(self, uin: int) -> Friend | None:
    /// ```
    ///
    /// [`Friend`]: friend::Friend
    pub fn get_friend<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let friend_list = client.get_friend_list_cached().await?;
            let friend = friend_list.find_friend(uin);
            Ok(match friend {
                Some(friend) => Some(py_obj(friend)?),
                None => None,
            })
        })
    }

    /// 获取群。
    ///
    /// 参考 [`Group`]。
    ///
    /// # Examples
    /// ```python
    /// group = await client.get_group(12345678)
    /// print(group.name)
    /// ```
    ///
    /// # Python
    /// ```python
    /// async def get_group(self, group_id: int) -> Group: ...
    /// ```
    pub fn get_group<'py>(&self, py: Python<'py>, group_id: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            if let Some(info) = client.inner().get_group_info(group_id).await? {
                let group = Group { client, info };
                Ok(Some(py_obj(group)?))
            } else {
                Ok(None)
            }
        })
    }

    /// 批量获取群，返回 `{ 群号: 群对象 }` 的字典。
    ///
    /// 当给出的群号不存在，或者未加入这个群时，将不会在返回值中出现。这意味着返回值长度可能会小于参数长度。
    ///
    /// 参考 [`Group`]。
    ///
    /// # Examples
    /// ```python
    /// groups = await client.get_groups([12345678, 87654321])
    /// if 12345678 in groups:
    ///     print(groups[12345678].name)
    /// else:
    ///     print("未加入群 12345678 或群不存在")
    /// ```
    ///
    /// # Python
    /// ```python
    /// async def get_groups(self, group_ids: Sequence[int]) -> dict[int, Group]: ...
    /// ```
    pub fn get_groups<'py>(&self, py: Python<'py>, group_ids: Vec<i64>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let infos = client.inner().get_group_infos(group_ids).await?;
            let infos = infos.into_iter().map(|info| (info.code, info));
            Ok(Python::with_gil(|py| -> PyResult<PyObject> {
                let dict = PyDict::new(py);
                for (key, info) in infos {
                    let group = Group {
                        client: client.clone(),
                        info,
                    };
                    dict.set_item(key, PyCell::new(py, group)?)?;
                }
                Ok(dict.into_py(py))
            })?)
        })
    }

    /// 获取群列表。
    ///
    /// 参考 [`Group`]。
    ///
    /// # Examples
    /// ```python
    /// group_list = await client.get_group_list()
    /// for group in group_list:
    ///     print(group.name)
    /// ```
    ///
    /// # Note
    /// 此方法获取到的 `last_msg_seq` 不可用，如需要此字段请使用 [`get_group`] 或 [`get_groups`]。
    ///
    /// # Python
    /// ```python
    /// async def get_group_list() -> list[Group]: ...
    /// ```
    ///
    /// [`get_group`]: crate::client::Client::get_group
    /// [`get_groups`]: crate::client::Client::get_groups
    pub fn get_group_list<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let group_list = client.inner().get_group_list().await?;
            let group_list = group_list.into_iter().map(|info| Group {
                client: client.clone(),
                info,
            });
            Ok(Python::with_gil(|py| -> PyResult<PyObject> {
                let list = PyList::new(
                    py,
                    group_list
                        .map(|info| PyCell::new(py, info))
                        .collect::<Result<Vec<_>, _>>()?,
                );
                Ok(list.into_py(py))
            })?)
        })
    }
}
