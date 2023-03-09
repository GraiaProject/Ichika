mod cached;
mod friend;
mod group;
mod structs;
use std::sync::Arc;

use cached::cache;
use group::Group;
use pyo3::prelude::*;
use pyo3::types::*;
use structs::*;
use tokio::task::JoinHandle;

use crate::login::{reconnect, TokenRW};
use crate::message::convert::extract_message_chain;
use crate::utils::{py_future, py_none, py_use, AsPython};
#[pyclass(subclass)]
pub struct PlumbingClient {
    client: Arc<ricq::client::Client>,
    alive: Option<JoinHandle<()>>,
    #[pyo3(get)]
    uin: i64,
    token_rw: TokenRW,
}

/// 用于向 Python 内的 `ichika.client.Client` 传递初始值
#[pyclass]
#[derive(Clone)]
pub struct ClientInitializer {
    pub uin: i64,
    pub client: Arc<ricq::Client>,
    pub alive: Arc<std::sync::Mutex<Option<JoinHandle<()>>>>,
    pub token_rw: TokenRW,
}

#[pymethods]
impl PlumbingClient {
    #[new]
    pub fn new(init: ClientInitializer) -> Self {
        Self {
            client: init.client,
            alive: init.alive.lock().unwrap().take(),
            uin: init.uin,
            token_rw: init.token_rw,
        }
    }

    pub fn keep_alive<'py>(&mut self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        let token_rw = self.token_rw.clone();
        let alive = self.alive.take();
        let uin = self.uin;
        py_future(py, async move {
            if let Some(mut alive) = alive {
                loop {
                    alive.await?;

                    // 断线重连
                    if let Some(handle) = reconnect(&client, &token_rw).await? {
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

    pub fn stop<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client.stop(ricq::client::NetworkStatus::Stop);
            Ok(())
        })
    }

    pub fn get_account_info<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let info = client.account_info.read().await;
            Ok(AccountInfo {
                nickname: info.nickname.clone(),
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
        let client = self.client.clone();
        py_future(py, async move {
            let friend_list = cache(client).await.fetch_friend_list().await?;
            Ok((*friend_list).clone().obj())
        })
    }

    pub fn get_friend_list_raw<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let mut cache = cache(client).await;
            cache.flush_friend_list().await;
            let friend_list = cache.fetch_friend_list().await?;
            Ok((*friend_list).clone().obj())
        })
    }

    pub fn get_friends<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let friend_list = cache(client).await.fetch_friend_list().await?;
            Ok(Python::with_gil(|py| friend_list.friends(py)))
        })
    }

    pub fn find_friend<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let friend_list = cache(client).await.fetch_friend_list().await?;
            Ok(friend_list.find_friend(uin))
        })
    }

    pub fn poke_friend<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client.friend_poke(uin).await?;
            Ok(())
        })
    }
}

#[pymethods]
impl PlumbingClient {
    pub fn get_group<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let group = cache(client).await.fetch_group(uin).await?;
            Ok((*group).clone().obj())
        })
    }

    pub fn get_group_raw<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let mut cache = cache(client).await;
            cache.flush_group(uin).await;
            let group = cache.fetch_group(uin).await?;
            Ok((*group).clone().obj())
        })
    }

    pub fn find_group<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let group = client.get_group_info(uin).await?;
            Ok(group.map(Group::from))
        })
    }

    pub fn get_groups<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let infos = client.get_group_list().await?;
            Ok(py_use(|py| {
                PyTuple::new(
                    py,
                    infos
                        .into_iter()
                        .map(|g| Group::from(g).obj())
                        .collect::<Vec<PyObject>>(),
                )
                .obj()
            }))
        })
    }
}

#[pymethods]
impl PlumbingClient {
    pub fn get_member<'py>(
        &self,
        py: Python<'py>,
        group_uin: i64,
        uin: i64,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let member = cache(client).await.fetch_member(group_uin, uin).await?;
            Ok((*member).clone().obj())
        })
    }

    pub fn get_member_raw<'py>(
        &self,
        py: Python<'py>,
        group_uin: i64,
        uin: i64,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let mut cache = cache(client).await;
            cache.flush_member(group_uin, uin).await;
            let member = cache.fetch_member(group_uin, uin).await?;
            Ok((*member).clone().obj())
        })
    }

    pub fn poke_member<'py>(
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
    pub fn send_friend_message<'py>(
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

    pub fn send_group_message<'py>(
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
