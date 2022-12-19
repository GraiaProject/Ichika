use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::utils::{py_future, retry};
use crate::{events::PyHandler, import_call};
use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use futures_util::StreamExt;
use pyo3::{exceptions::PyValueError, prelude::*, types::PyList};
use pythonize::*;
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
async fn load_device_json(data_folder: PathBuf) -> Result<Device> {
    let mut device_ricq_json = data_folder.clone();
    device_ricq_json.push("ricq.device.json");

    // 解析设备信息
    let device = if device_ricq_json.exists() {
        // 尝试读取已有的 `ricq.device.json`
        tracing::info!("发现 `ricq.device.json`, 读取");
        let json = tokio::fs::read_to_string(device_ricq_json).await?;
        serde_json::from_str::<Device>(json.as_str())?
    } else {
        // 如果 `device.json` 存在那就尝试转换
        let mut device_json = data_folder.clone();
        device_json.push("device.json");
        let mut device: Option<Device> = None;
        if device_json.exists() {
            tracing::info!("发现 `device.json`, 尝试转换");
            let json_data = tokio::fs::read_to_string(device_json).await?;
            match Python::with_gil(move |py| -> Result<Device, PythonizeError> {
                // data = json.loads(json_data)
                let data = import_call!(py, "json" => "loads" => json_data)?;
                // device_dc = ichika.scripts.device.converter.convert(data)
                let device_dc =
                    import_call!(py, "ichika.scripts.device.converter" => "convert" => data)?;
                // converted = dataclasses.asdict(device_dc)
                let converted = import_call!(py, "dataclasses" => "asdict" => device_dc)?;
                depythonize(converted)
            }) {
                Ok(d) => {
                    device = Some(d);
                }
                Err(err) => {
                    tracing::error!("转换 `device.json` 发生错误: {}", err);
                    tracing::info!("重新创建 `device.ricq.json`")
                }
            }
        } else {
            tracing::info!("未找到 `device.ricq.json`, 正在创建")
        }
        let device: Device = match device {
            Some(device) => device,
            None => Python::with_gil(|py| -> Result<Device, PythonizeError> {
                let device_dc =
                    import_call!(py, "ichika.scripts.device.generator" => "generate" => @tuple ())?;
                let converted = import_call!(py, "dataclasses" => "asdict" => device_dc)?;
                depythonize(converted)
            })?,
        };
        let json = serde_json::to_string::<Device>(&device)?;
        tokio::fs::write(device_ricq_json, json).await?;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenWithProtocol {
    #[serde(default = "String::new")]
    protocol: String,
    #[serde(flatten)]
    token: Token,
}

async fn try_token_login(
    client: &Client,
    protocol: &Protocol,
    mut data_folder: PathBuf,
) -> Result<bool> {
    let token_path = {
        data_folder.push("token.json");
        data_folder
    };
    if !token_path.exists() {
        return Ok(false);
    }
    tracing::info!("发现上一次登录的 token，尝试使用 token 登录");
    let token = tokio::fs::read_to_string(&token_path).await?;
    let token: TokenWithProtocol = serde_json::from_str(&token)?;
    if format!("{:?}", protocol) == token.protocol {
        match client.token_login(token.token).await {
            Ok(login_resp) => {
                if let LoginResponse::Success(LoginSuccess {
                    ref account_info, ..
                }) = login_resp
                {
                    tracing::info!("登录成功: {:?}", account_info);
                    return Ok(true);
                }
                tracing::error!("登录失败：{:?}", login_resp);
            }
            Err(e) => {
                tracing::error!("token 登录失败：{:?}", e);
            }
        }
    } else {
        tracing::info!("登录协议与 token 协议不一致！");
    }
    tracing::info!("删除 token 重新登录...");
    tokio::fs::remove_file(token_path).await?;
    Ok(false)
}

async fn save_token(client: &Client, protocol: &Protocol, mut data_folder: PathBuf) -> Result<()> {
    let token = client.gen_token().await;
    let token = serde_json::to_string(&TokenWithProtocol {
        protocol: format!("{:?}", protocol),
        token,
    })?;
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
        method: &'py PyAny,
    ) -> PyResult<&'py PyAny> {
        match pythonize::depythonize::<LoginMethod>(&method) {
            Ok(method) => {
                let protocol = self_t.protocol.clone();
                let mut data_folder = self_t.data_folder.clone();
                let uin = self_t.uin;
                let handler = PyHandler::new(self_t.event_callbacks.clone_ref(py));
                py_future(py, async move {
                    data_folder.push(uin.to_string());
                    tokio::fs::create_dir_all(&data_folder).await?;

                    let device = load_device_json(data_folder.clone()).await?;
                    let (client, alive) = prepare_client(device, protocol.clone(), handler).await?;

                    if !try_token_login(&client, &protocol, data_folder.clone()).await? {
                        match method {
                            LoginMethod::Password(p) => {
                                password_login(&client, uin, p.password, p.md5, p.sms).await?;
                            }
                            LoginMethod::QRCode => {
                                qrcode_login(&client, uin).await?;
                            }
                        }
                    }

                    // 注册客户端，启动心跳。
                    after_login(&client).await;
                    save_token(&client, &protocol, data_folder.clone()).await?;
                    let init = crate::client::ClientInitializer {
                        uin: client.uin().await,
                        client,
                        alive: Arc::new(::std::sync::Mutex::new(Some(alive))),
                        data_folder,
                    };
                    Python::with_gil(|py| {
                        Ok(import_call!(py, "ichika.client" => "Client" => init)?.into_py(py))
                    })
                })
            }
            Err(e) => {
                return Err(PyValueError::new_err(format!("{:?}", e)));
            }
        }
    }
}
