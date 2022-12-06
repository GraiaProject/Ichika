use std::{path::PathBuf, sync::Arc};

use crate::login::reconnect;
use crate::utils::{py_future, py_none};
use pyo3::prelude::*;
use tokio::task::JoinHandle;
#[pyclass]
pub struct Client {
    client: Arc<ricq::client::Client>,
    alive: Option<JoinHandle<()>>,
    #[pyo3(get)]
    uin: i64,
    data_folder: PathBuf,
}

impl Client {
    #[allow(dead_code)]
    async fn new(client: Arc<ricq::Client>, alive: JoinHandle<()>, data_folder: PathBuf) -> Self {
        let uin = client.uin().await;
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

    #[getter]
    pub fn info<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let info = &*client.account_info.read().await;
            Ok(crate::client::structs::AccountInfo {
                nickname: info.nickname.clone(),
                age: info.age,
                gender: info.gender,
            })
        })
    }
}
