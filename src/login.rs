use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::events::PyHandler;
use crate::utils::{py_future, retry};
use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use futures_util::StreamExt;
use pyo3::{exceptions::PyValueError, prelude::*, types::PyList};
use ricq::{
    client::{Connector, DefaultConnector, NetworkStatus, Token},
    ext::{
        common::after_login,
        reconnect::{fast_login, Credential},
    },
    version::get_version,
    Client, Device, LoginDeviceLocked, LoginNeedCaptcha, LoginResponse, LoginSuccess,
    LoginUnknownStatus, Protocol,
};
use ricq::{QRCodeConfirmed, QRCodeImageFetch, QRCodeState};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::codec::{FramedRead, LinesCodec};

/// 加载 `device.json`。
async fn load_device_json(uin: i64, mut data_folder: PathBuf) -> Result<Device> {
    use crate::device;

    // 获取 `device.json` 的路径
    let device_json = {
        data_folder.push("device.json");
        data_folder
    };

    // 解析设备信息
    let device = if device_json.exists() {
        // 尝试读取已有的 `device.json`
        let json = tokio::fs::read_to_string(device_json).await?;
        device::from_json(&json, &device::random_from_uin(uin))?
    } else {
        // 否则，生成一个新的 `device.json` 并保存到文件中
        let device = device::random_from_uin(uin);
        let json = device::to_json(&device)?;
        tokio::fs::write(device_json, json).await?;
        device
    };

    Ok(device)
}

/// 创建客户端，准备登录。
async fn prepare_client(
    device: Device,
    protocol: Protocol,
    handler: PyHandler,
) -> Result<(Arc<Client>, JoinHandle<()>)> {
    let client = Arc::new(Client::new(device, get_version(protocol), handler));
    let alive = tokio::spawn({
        let client = client.clone();
        // 连接最快的服务器
        let stream = DefaultConnector.connect(&client).await?;
        async move { client.start(stream).await }
    });

    tokio::task::yield_now().await; // 等一下，确保连上了
    Ok((client, alive))
}

async fn try_token_login(client: &Client, mut data_folder: PathBuf) -> Result<bool> {
    let token_path = {
        data_folder.push("token.json");
        data_folder
    };
    if !token_path.exists() {
        return Ok(false);
    }
    tracing::info!("发现上一次登录的 token，尝试使用 token 登录");
    let token = tokio::fs::read_to_string(&token_path).await?;
    let token: Token = serde_json::from_str(&token)?;
    match client.token_login(token).await {
        Ok(login_resp) => {
            if let LoginResponse::Success(LoginSuccess {
                ref account_info, ..
            }) = login_resp
            {
                tracing::info!("登录成功: {:?}", account_info);
                return Ok(true);
            }
            bail!("登录失败，原因未知：{:?}", login_resp)
        }
        Err(_) => {
            tracing::info!("token 登录失败，将删除 token");
            tokio::fs::remove_file(token_path).await?;
            Ok(false)
        }
    }
}

async fn save_token(client: &Client, mut data_folder: PathBuf) -> Result<()> {
    let token = client.gen_token().await;
    let token = serde_json::to_string(&token)?;
    let token_path = {
        data_folder.push("token.json");
        data_folder
    };
    tokio::fs::write(token_path, token).await?;
    Ok(())
}

async fn password_login(
    client: &Client,
    uin: i64,
    password: String,
    md5: bool,
    sms: bool,
) -> Result<()> {
    tracing::info!("使用密码登录，uin={}", uin);

    let mut resp = if !md5 {
        client.password_login(uin, &password).await?
    } else {
        client
            .password_md5_login(uin, &hex::decode(password)?)
            .await?
    };

    loop {
        match resp {
            LoginResponse::Success(LoginSuccess {
                ref account_info, ..
            }) => {
                tracing::info!("登录成功: {:?}", account_info);
                break;
            }
            LoginResponse::DeviceLocked(LoginDeviceLocked {
                ref verify_url,
                ref message,
                ..
            }) => {
                if sms {
                    // resp = client.request_sms().await.expect("无法请求短信验证码");
                    bail!("暂不支持短信登录")
                } else {
                    tracing::info!("设备锁: {}", message.as_deref().unwrap_or(""));
                    tracing::info!("验证 url: {}", verify_url.as_deref().unwrap_or(""));
                    bail!("手机打开 url，处理完成后重启程序")
                }
            }
            LoginResponse::NeedCaptcha(LoginNeedCaptcha { ref verify_url, .. }) => {
                tracing::info!("滑块 url: {}", verify_url.as_deref().unwrap_or("")); // TODO: 接入 TxCaptchaHelper
                tracing::info!("请输入 ticket:");
                let mut reader = FramedRead::new(tokio::io::stdin(), LinesCodec::new());
                let ticket = reader.next().await.transpose().unwrap().unwrap();
                resp = client.submit_ticket(&ticket).await?;
            }
            LoginResponse::DeviceLockLogin { .. } => {
                resp = client.device_lock_login().await?;
            }
            LoginResponse::AccountFrozen => bail!("账号被冻结"),
            LoginResponse::TooManySMSRequest => bail!("短信请求过于频繁"),
            LoginResponse::UnknownStatus(LoginUnknownStatus {
                ref status,
                ref tlv_map,
                ref message,
            }) => {
                bail!("登陆失败，原因未知：{}, {}, {:?}", status, message, tlv_map);
            }
        }
    }

    Ok(())
}

pub(crate) async fn reconnect(
    client: &Arc<Client>,
    data_folder: &Path,
) -> Result<Option<JoinHandle<()>>> {
    retry(
        10,
        || async {
            // 如果不是网络原因掉线，不重连（服务端强制下线/被踢下线/用户手动停止）
            if client.get_status() != (NetworkStatus::NetworkOffline as u8) {
                tracing::warn!("客户端因非网络原因下线，不再重连");
                return Ok(None);
            }
            client.stop(NetworkStatus::NetworkOffline);

            tracing::error!("客户端连接中断，将在 10 秒后重连");
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;

            let alive = tokio::spawn({
                let client = client.clone();
                // 连接最快的服务器
                let stream = DefaultConnector.connect(&client).await?;
                async move { client.start(stream).await }
            });
            tokio::task::yield_now().await; // 等一下，确保连上了

            // 启动接收后，再发送登录请求，否则报错 NetworkError
            let token_path = data_folder.join("token.json");
            if !token_path.exists() {
                tracing::error!("重连失败：未找到上次登录的 token");
                return Ok(None);
            }
            let token = tokio::fs::read_to_string(token_path).await?;
            let token = match serde_json::from_str(&token) {
                Ok(token) => token,
                Err(err) => {
                    tracing::error!("重连失败：无法解析上次登录的 token，{}", err);
                    return Ok(None);
                }
            };
            fast_login(client, &Credential::Token(token))
                .await
                .map_err(|e| {
                    client.stop(NetworkStatus::NetworkOffline);
                    e
                })?;

            after_login(client).await;

            tracing::info!("客户端重连成功");
            Ok(Some(alive))
        },
        |e, c| async move {
            let backtrace = e.backtrace();
            tracing::error!("客户端重连失败，原因：{}，剩余尝试 {} 次", e, c);
            tracing::debug!("backtrace: {}", backtrace);
        },
    )
    .await
}

pub(super) fn print_qrcode(qrcode: &Bytes) -> Result<String> {
    let qrcode = image::load_from_memory(qrcode)?.to_luma8();
    let mut qrcode = rqrr::PreparedImage::prepare(qrcode);
    let grids = qrcode.detect_grids();
    if grids.len() != 1 {
        return Err(anyhow!("无法识别二维码"));
    }
    let (_, content) = grids[0].decode()?;
    let qrcode = qrcode::QrCode::new(content)?;
    let qrcode = qrcode.render::<qrcode::render::unicode::Dense1x2>().build();
    Ok(qrcode)
}

pub(super) async fn qrcode_login(client: &Client, uin: i64) -> Result<()> {
    tracing::info!("使用二维码登录，uin={}", uin);

    let mut resp = client.fetch_qrcode().await?;

    let mut image_sig = Bytes::new();
    loop {
        match resp {
            QRCodeState::ImageFetch(QRCodeImageFetch {
                ref image_data,
                ref sig,
            }) => {
                let qr = print_qrcode(image_data)?;
                tracing::info!("请扫描二维码: \n{}", qr);
                image_sig = sig.clone();
            }
            QRCodeState::WaitingForScan => {
                tracing::debug!("等待二维码扫描")
            }
            QRCodeState::WaitingForConfirm => {
                tracing::debug!("二维码已扫描，等待确认")
            }
            QRCodeState::Timeout => {
                tracing::info!("二维码已超时，重新获取");
                if let QRCodeState::ImageFetch(QRCodeImageFetch {
                    ref image_data,
                    ref sig,
                }) = client.fetch_qrcode().await.expect("failed to fetch qrcode")
                {
                    let qr = print_qrcode(image_data)?;
                    tracing::info!("请扫描二维码: \n{}", qr);
                    image_sig = sig.clone();
                }
            }
            QRCodeState::Confirmed(QRCodeConfirmed {
                ref tmp_pwd,
                ref tmp_no_pic_sig,
                ref tgt_qr,
                ..
            }) => {
                tracing::info!("二维码已确认");
                let mut login_resp = client.qrcode_login(tmp_pwd, tmp_no_pic_sig, tgt_qr).await?;
                if let LoginResponse::DeviceLockLogin { .. } = login_resp {
                    login_resp = client.device_lock_login().await?;
                }
                if let LoginResponse::Success(LoginSuccess {
                    ref account_info, ..
                }) = login_resp
                {
                    tracing::info!("登录成功: {:?}", account_info);
                    let real_uin = client.uin().await;
                    if real_uin != uin {
                        bail!("预期登录账号 {}，但实际登陆账号为 {}", uin, real_uin);
                    }
                    break;
                }
                bail!("登录失败，原因未知：{:?}", login_resp)
            }
            QRCodeState::Canceled => {
                bail!("二维码已取消")
            }
        }
        sleep(Duration::from_secs(5)).await;
        resp = client.query_qrcode_result(&image_sig).await?;
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum LoginMethod {
    Password(Password),
    QRCode,
}

#[derive(Debug, Serialize, Deserialize)]
struct Password {
    password: String,
    md5: bool,
    sms: bool,
}

#[pyclass]
pub struct Account {
    protocol: Protocol,
    uin: i64,
    data_folder: PathBuf,
    #[pyo3(get)]
    event_callbacks: Py<PyList>,
}

#[pymethods]
impl Account {
    #[new]
    #[args(protocol = "\"ipad\".to_string()")]
    fn new(py: Python, uin: i64, data_folder: PathBuf, mut protocol: String) -> PyResult<Self> {
        protocol.make_ascii_lowercase();
        let protocol = match protocol.as_str() {
            "ipad" => Protocol::IPad,
            "android" | "android_phone" => Protocol::AndroidPhone,
            "watch" | "android_watch" => Protocol::AndroidWatch,
            "mac" | "macos" => Protocol::MacOS,
            "qidian" => Protocol::QiDian,
            _ => Err(anyhow!("不支持的协议"))?,
        };
        let cbs = PyList::empty(py).into_py(py);
        Ok(Self {
            protocol,
            uin,
            data_folder,
            event_callbacks: cbs,
        })
    }

    pub fn login<'py>(
        self_t: PyRef<'py, Self>,
        py: Python<'py>,
        method: String,
    ) -> PyResult<&'py PyAny> {
        match serde_json::from_str::<LoginMethod>(&method) {
            Ok(method) => {
                let protocol = self_t.protocol.clone();
                let mut data_folder = self_t.data_folder.clone();
                let uin = self_t.uin;
                let handler = PyHandler::new(self_t.event_callbacks.clone_ref(py));
                py_future(py, async move {
                    data_folder.push(uin.to_string());
                    tokio::fs::create_dir_all(&data_folder).await?;

                    let device = load_device_json(uin, data_folder.clone()).await?;
                    let (client, alive) = prepare_client(device, protocol, handler).await?;

                    match method {
                        LoginMethod::Password(p) => {
                            if !try_token_login(&client, data_folder.clone()).await? {
                                password_login(&client, uin, p.password, p.md5, p.sms).await?;
                            }
                        }
                        LoginMethod::QRCode => {
                            if !try_token_login(&client, data_folder.clone()).await? {
                                qrcode_login(&client, uin).await?;
                            }
                        }
                    }

                    // 注册客户端，启动心跳。
                    after_login(&client).await;
                    save_token(&client, data_folder.clone()).await?;
                    Ok(crate::client::Client::new(client, alive, data_folder).await)
                })
            }
            Err(e) => {
                return Err(PyValueError::new_err(format!("{:?}", e)));
            }
        }
    }
}
