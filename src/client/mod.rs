mod friend;
mod group;
mod structs;
mod utils;
use std::time::Duration;
use std::{path::PathBuf, sync::Arc};

use crate::login::reconnect;
use crate::message::convert::extract_message_chain;
use crate::py_intern;
use crate::utils::{py_future, py_none};
use friend::FriendList;
use group::Group;
use pyo3::{prelude::*, types::*};
use structs::*;
use tokio::task::JoinHandle;
use utils::CacheField;
#[pyclass(subclass)]
pub struct PlumbingClient {
    client: Arc<ricq::client::Client>,
    alive: Option<JoinHandle<()>>,
    #[pyo3(get)]
    uin: i64,
    data_folder: PathBuf,
    friend_cache: Arc<CacheField<FriendList>>,
}

/// 用于向 Python 内的 `ichika.client.Client` 传递初始值
#[pyclass]
#[derive(Clone)]
pub struct ClientInitializer {
    pub uin: i64,
    pub client: Arc<ricq::Client>,
    pub alive: Arc<::std::sync::Mutex<Option<JoinHandle<()>>>>,
    pub data_folder: PathBuf,
}

#[pymethods]
impl PlumbingClient {
    #[new]
    pub fn new(init: ClientInitializer) -> Self {
        Self {
            client: init.client,
            alive: init.alive.lock().unwrap().take(),
            uin: init.uin,
            data_folder: init.data_folder,
            friend_cache: Arc::new(CacheField::new(Duration::from_secs(3600))),
        }
    }

    pub fn keep_alive<'py>(&mut self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        let data_folder = self.data_folder.clone();
        let alive = self.alive.take();
        let uin = self.uin;
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
            tracing::info!("客户端 {} 连接断开", uin);
            Ok(py_none())
        })
    }

    #[getter]
    pub fn online(&self) -> bool {
        self.client
            .online
            .load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn get_account_info<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let info = client.account_info.read().await;
            Ok(AccountInfo {
                nickname: py_intern!(&info.nickname),
                age: info.age,
                gender: info.gender,
            })
        })
    }

    pub fn get_other_clients<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let other_clients = &*client.online_clients.read().await;
            Python::with_gil(|py| {
                let tup: PyObject = PyTuple::new(
                    py,
                    other_clients
                        .iter()
                        .map(|cl| {
                            OtherClientInfo {
                                app_id: cl.app_id,
                                instance_id: cl.instance_id,
                                sub_platform: cl.sub_platform.clone(),
                                device_kind: cl.device_kind.clone(),
                            }
                            .into_py(py)
                        })
                        .collect::<Vec<PyObject>>(),
                )
                .into_py(py);
                Ok(tup)
            })
        })
    }
}

#[pymethods]
impl PlumbingClient {
    pub fn get_friend_list<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let field = self.friend_cache.clone();
        let client = self.client.clone();
        py_future(py, async move {
            let friend_list = field.get(client).await?;
            Ok(friend_list)
        })
    }

    pub fn get_friend_list_raw<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let field = self.friend_cache.clone();
        let client = self.client.clone();
        py_future(py, async move {
            field.clear().await;
            let friend_list = field.get(client).await?;
            Ok(friend_list)
        })
    }

    pub fn get_friends<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let field = self.friend_cache.clone();
        let client = self.client.clone();
        py_future(py, async move {
            let friend_list = field.get(client).await?;
            Ok(Python::with_gil(|py| friend_list.friends(py)))
        })
    }

    pub fn find_friend<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        let field = self.friend_cache.clone();
        let client = self.client.clone();
        py_future(py, async move {
            let friend_list = field.get(client).await?;
            Ok(friend_list.find_friend(uin))
        })
    }

    pub fn poke_friend_raw<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client.friend_poke(uin).await?;
            Ok(())
        })
    }
}
#[pymethods]
impl PlumbingClient {
    pub fn find_group<'py>(&self, py: Python<'py>, group_uin: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            if let Some(info) = client.get_group_info(group_uin).await? {
                Ok(Some(Group::from(info)))
            } else {
                Ok(None)
            }
        })
    }
    pub fn find_groups<'py>(&self, py: Python<'py>, group_uins: Vec<i64>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let infos = client.get_group_infos(group_uins).await?;
            let infos = infos.into_iter().map(|info| (info.code, info));
            Ok(Python::with_gil(|py| -> PyResult<PyObject> {
                let dict = PyDict::new(py);
                for (key, info) in infos {
                    dict.set_item(key, Group::from(info).into_py(py))?;
                }
                Ok(dict.into_py(py))
            })?)
        })
    }
    pub fn get_groups<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let infos = client.get_group_list().await?;
            Python::with_gil(|py| {
                let tup: PyObject = PyTuple::new(
                    py,
                    infos
                        .into_iter()
                        .map(|info| Group::from(info))
                        .map(|g| g.into_py(py))
                        .collect::<Vec<PyObject>>(),
                )
                .into_py(py);
                Ok(tup)
            })
        })
    }

    pub fn poke_member_raw<'py>(
        &self,
        py: Python<'py>,
        group_uin: i64,
        member_uin: i64,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client.group_poke(group_uin, member_uin).await?;
            Ok(())
        })
    }
}

#[pymethods]
impl PlumbingClient {
    pub fn send_friend_message_raw<'py>(
        &self,
        py: Python<'py>,
        uin: i64,
        chain: &'py PyList,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        let chain = extract_message_chain(chain)?;
        py_future(py, async move {
            // TODO: Audio
            let ricq::structs::MessageReceipt { seqs, rands, time } =
                client.send_friend_message(uin, chain).await?;
            Ok(Python::with_gil(|py| RawMessageReceipt {
                seqs: PyTuple::new(py, seqs).into_py(py),
                rands: PyTuple::new(py, rands).into_py(py),
                time,
            }))
        })
    }

    pub fn send_group_message_raw<'py>(
        &self,
        py: Python<'py>,
        group_uin: i64,
        chain: &'py PyList,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        let chain = extract_message_chain(chain)?;
        py_future(py, async move {
            // TODO: Audio
            let ricq::structs::MessageReceipt { seqs, rands, time } =
                client.send_group_message(group_uin, chain).await?;
            Ok(Python::with_gil(|py| RawMessageReceipt {
                seqs: PyTuple::new(py, seqs).into_py(py),
                rands: PyTuple::new(py, rands).into_py(py),
                time,
            }))
        })
    }
}
