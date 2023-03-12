use std::sync::Arc;

use pyo3::exceptions::*;
use pyo3::prelude::*;
use pyo3::types::*;
use pythonize::depythonize;
use ricq::client::{Client, Connector, DefaultConnector, NetworkStatus, Token};
use ricq::ext::common::after_login;
use ricq::version::get_version;
use ricq::{
    Device,
    LoginDeviceLocked,
    LoginNeedCaptcha,
    LoginResponse,
    LoginSuccess,
    LoginUnknownStatus,
    Protocol,
    QRCodeState,
};
use tokio::task::JoinHandle;

use crate::events::PyHandler;
use crate::exc::MapPyErr;
use crate::utils::{partial, py_bytes, py_future, py_try, py_use};
use crate::{exc, import_call, PyRet};

async fn prepare_client(
    device: Device,
    protocol: Protocol,
    handler: PyHandler,
) -> PyResult<(Arc<Client>, JoinHandle<()>)> {
    let client = Arc::new(Client::new(device, get_version(protocol), handler));
    let alive = tokio::spawn({
        let client = client.clone();
        // 连接最快的服务器
        let stream = DefaultConnector
            .connect(&client)
            .await
            .map_err(|e| PyIOError::new_err(e.to_string()))?;

        #[allow(
            clippy::redundant_async_block,
            reason = "FP: rust-lang/rust-clippy#10482"
        )]
        async move {
            client.start(stream).await;
        }
    });

    tokio::task::yield_now().await; // 等一下，确保连上了
    Ok((client, alive))
}

#[derive(FromPyObject)]
pub enum PasswordCredential {
    #[pyo3(transparent, annotation = "str")]
    String(Py<PyString>),
    #[pyo3(transparent, annotation = "bytes")]
    MD5(Py<PyBytes>),
}

fn protocol_from_str(protocol: &str) -> PyResult<Protocol> {
    match protocol {
        "IPad" => Ok(Protocol::IPad),
        "AndroidPhone" => Ok(Protocol::AndroidPhone),
        "AndroidWatch" => Ok(Protocol::AndroidWatch),
        "MacOS" => Ok(Protocol::MacOS),
        "QiDian" => Ok(Protocol::QiDian),
        _ => Err(exc::LoginError::new_err("未知协议")),
    }
}

#[derive(Debug, Clone)]
pub struct TokenRW {
    get_token: PyObject,
    write_token: PyObject,
}

impl TokenRW {
    fn get(&self) -> PyResult<Option<Token>> {
        py_try(|py| {
            let mut token: Option<Token> = None;
            let py_token = self.get_token.as_ref(py).call0()?;
            if !py_token.is_none() {
                token = serde_json::from_slice(
                    py_token
                        .downcast::<PyBytes>()
                        .map_err(|e| PyTypeError::new_err(format!("token 类型不是 bytes: {e:?}")))?
                        .as_bytes(),
                )
                .map_err(|e| {
                    exc::LoginError::new_err(format!("无法转换 token 为 RICQ Token: {e:?}"))
                })?;
            }
            Ok(token)
        })
    }

    async fn set(&self, client: &Client) -> PyResult<()> {
        let token = client.gen_token().await;
        let token = serde_json::to_vec::<Token>(&token)
            .map_err(|e| exc::RICQError::new_err(format!("{e:?}")))?;
        py_try(|py| self.write_token.call1(py, (py_bytes(&token),)))?;
        Ok(())
    }

    async fn try_login(&self, client: &Client) -> PyResult<bool> {
        let token = self.get()?;
        if let Some(token) = token {
            match client.token_login(token).await {
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
            tracing::info!("未能找到已有 token，重新登录");
        }
        Ok(false)
    }
}

fn parse_login_args<'py>(
    py: Python<'py>,
    uin: i64,
    protocol: &'py PyAny,
    store: &'py PyAny,
    event_callbacks: &'py PyList,
) -> PyResult<(Protocol, PyHandler, Device, TokenRW)> {
    let handler = PyHandler::new(event_callbacks.into_py(py));

    let get_token = partial(py).call1((store.getattr("get_token")?, uin, protocol))?;
    let write_token = partial(py).call1((store.getattr("write_token")?, uin, protocol))?;

    let device = store.getattr("get_device")?.call1((uin, protocol))?; // JSON
    let device: Device = depythonize(device)
        .map_err(|e| exc::LoginError::new_err(format!("无法解析传入的 Device: {e:?}")))?;

    // Extract Protocol
    let protocol = protocol.getattr("value")?.extract::<String>()?;
    let protocol = protocol_from_str(&protocol)?;

    Ok((
        protocol,
        handler,
        device,
        TokenRW {
            get_token: get_token.into_py(py),
            write_token: write_token.into_py(py),
        },
    ))
}

fn call_state(py: Python, getter: &PyObject, name: &str, args: impl IntoPy<Py<PyTuple>>) -> PyRet {
    let handler = getter.as_ref(py).call1((name,))?;
    if handler.is_none() {
        return Ok(py.None()); // return None
    }
    Ok(handler.call1(args)?.into_py(py))
}

pub async fn reconnect(
    client: &Arc<Client>,
    token_rw: &TokenRW,
) -> PyResult<Option<JoinHandle<()>>> {
    crate::utils::py_retry(
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

                #[allow(
                    clippy::redundant_async_block,
                    reason = "FP: rust-lang/rust-clippy#10482"
                )]
                async move {
                    client.start(stream).await;
                }
            });
            tokio::task::yield_now().await; // 等一下，确保连上了

            // 启动接收后，再发送登录请求，否则报错 NetworkError
            if !token_rw.try_login(client).await? {
                client.stop(NetworkStatus::NetworkOffline);
                return Ok(None);
            }

            after_login(client).await;

            tracing::info!("客户端重连成功");
            Ok(Some(alive))
        },
        |e, c| async move {
            tracing::error!("客户端重连失败，原因：{}，剩余尝试 {} 次", e, c);
        },
    )
    .await
}

async fn make_password_login_req(
    uin: i64,
    client: &Client,
    credential: &PasswordCredential,
) -> ricq::RQResult<LoginResponse> {
    match credential {
        PasswordCredential::String(str) => {
            client
                .password_login(uin, &py_use(|py| str.as_ref(py).to_string()))
                .await
        }
        PasswordCredential::MD5(bytes) => {
            client
                .password_md5_login(uin, &py_use(|py| bytes.as_ref(py).as_bytes().to_owned()))
                .await
        }
    }
}
async fn handle_device_lock(
    data: &LoginDeviceLocked,
    uin: i64,
    client: &Client,
    credential: &PasswordCredential,
    handle_getter: PyObject,
    sms: bool,
) -> PyResult<LoginResponse> {
    let sms_phone = data.sms_phone.as_ref();
    let message = data
        .message
        .as_ref()
        .map_or_else(|| "请解锁设备锁进行验证", |msg| msg.as_str());
    let verify_url = data.verify_url.as_ref().map_or(
        Err(exc::RICQError::new_err("无法获取验证地址")),
        |url| Ok(url.clone()),
    )?;
    tracing::info!("{:?}", data.clone());
    if let Some(sms_phone) = sms_phone
        && sms
        && let Ok(rsp) = client.request_sms().await {
            if !matches!(rsp, LoginResponse::DeviceLocked(_)) {
                return Ok(rsp);
            }

            let sms_code = py_try(|py| {
                let res = call_state(py, &handle_getter, "RequestSMS", (message,sms_phone))?;
                let res = res.as_ref(py);
                if !res.is_none() {
                    return Ok(Some(res.extract::<String>()?));
                }
                Ok(None)
            })?;

            if let Some(sms_code) = sms_code {
                return client
                .submit_sms_code(&sms_code)
                .await
                .py_res();
            }
        }
    py_try(|py| call_state(py, &handle_getter, "DeviceLocked", (message, verify_url)))?;
    make_password_login_req(uin, client, credential)
        .await
        .py_res()
}

async fn password_login_process(
    client: &Client,
    uin: i64,
    credential: PasswordCredential,
    sms: bool,
    handle_getter: PyObject,
) -> PyResult<()> {
    let mut resp: LoginResponse = make_password_login_req(uin, client, &credential)
        .await
        .py_res()?;

    loop {
        match resp {
            LoginResponse::Success(LoginSuccess { .. }) => {
                py_try(|py| call_state(py, &handle_getter, "Success", ()))?;
                break;
            }
            LoginResponse::DeviceLocked(data) => {
                resp =
                    handle_device_lock(&data, uin, client, &credential, handle_getter.clone(), sms)
                        .await?;
            }
            LoginResponse::NeedCaptcha(LoginNeedCaptcha { ref verify_url, .. }) => {
                let verify_url = verify_url.as_ref().map_or(
                    Err(exc::RICQError::new_err("无法获取验证地址")),
                    |url| Ok(url.clone()),
                )?;
                let ticket = py_try(|py| {
                    Ok(
                        call_state(py, &handle_getter, "NeedCaptcha", (verify_url,))?
                            .downcast::<PyString>(py)?
                            .to_string(),
                    )
                })?;
                resp = client.submit_ticket(&ticket).await.py_res()?;
            }
            LoginResponse::DeviceLockLogin { .. } => {
                py_try(|py| call_state(py, &handle_getter, "DeviceLockLogin", ()))?;
                resp = client.device_lock_login().await.py_res()?;
            }
            LoginResponse::AccountFrozen => {
                py_try(|py| call_state(py, &handle_getter, "AccountFrozen", ()))?;
                break;
            }
            LoginResponse::TooManySMSRequest => {
                py_try(|py| call_state(py, &handle_getter, "TooManySMSRequest", ()))?;
                break;
            }
            LoginResponse::UnknownStatus(LoginUnknownStatus {
                ref status,
                ref message,
                ..
            }) => {
                let (status, message) = (*status, message.clone());
                py_try(|py| call_state(py, &handle_getter, "UnknownStatus", (message, status)))?;
                break;
            }
        }
    }

    Ok(())
}

async fn post_login(client: Arc<Client>, alive: JoinHandle<()>, token_rw: TokenRW) -> PyRet {
    after_login(&client).await;
    token_rw.set(&client).await?;
    let init = crate::client::ClientInitializer {
        uin: client.uin().await,
        client,
        alive: Arc::new(std::sync::Mutex::new(Some(alive))),
        token_rw,
    };
    py_try(|py| Ok(import_call!(py, "ichika.client" => "Client" => init)?.into_py(py)))
}

#[pyfunction]
#[allow(clippy::too_many_arguments, reason = "Required for Python binding")]
pub fn password_login<'py>(
    py: Python<'py>,
    uin: i64,
    credential: PasswordCredential,
    use_sms: bool,
    protocol: &'py PyAny,
    store: &'py PyAny,
    event_callbacks: &'py PyList, // List[Callable[...]]
    login_callbacks: PyObject,    // PasswordLoginCallbacks
) -> PyResult<&'py PyAny> {
    let (protocol, handler, device, token_rw) =
        parse_login_args(py, uin, protocol, store, event_callbacks)?;
    py_future(py, async move {
        let (client, alive) = prepare_client(device, protocol.clone(), handler).await?;
        if !token_rw.try_login(&client).await? {
            tracing::info!("正在使用密码登录 {}", uin);
            let handle_getter: PyObject = py_try(|py| login_callbacks.getattr(py, "get_handle"))?;
            password_login_process(&client, uin, credential, use_sms, handle_getter).await?;
        }

        Ok(post_login(client, alive, token_rw).await?)
    })
}

fn parse_qrcode(qrcode: &bytes::Bytes) -> PyResult<Vec<Vec<bool>>> {
    let qrcode = image::load_from_memory(qrcode)
        .map_err(|e| PyValueError::new_err(format!("加载二维码图像出现错误: {e:?}")))?
        .to_luma8();
    let mut qrcode = rqrr::PreparedImage::prepare(qrcode);
    let grids = qrcode.detect_grids();
    if grids.len() != 1 {
        return Err(PyValueError::new_err(format!(
            "无法识别二维码, 发现 {} 个二维码",
            grids.len()
        )));
    }
    let (_, content) = grids[0]
        .decode()
        .map_err(|e| PyValueError::new_err(format!("解码二维码出现错误: {e:?}")))?;
    let qrcode = qrcode::QrCode::new(content)
        .map_err(|e| PyValueError::new_err(format!("生成二维码数据出现错误: {e:?}")))?;

    let width = qrcode.width();
    Ok(qrcode
        .into_colors()
        .chunks(width)
        .map(|chunk| {
            chunk
                .iter()
                .map(|c| match c {
                    qrcode::Color::Light => true,
                    qrcode::Color::Dark => false,
                })
                .collect()
        })
        .collect())
}

async fn qrcode_login_process(
    client: &Client,
    decl_uin: i64,
    handle_getter: PyObject,
    interval: f64,
) -> PyResult<()> {
    let mut resp = client.fetch_qrcode().await.py_res()?;
    let mut image_sig = bytes::Bytes::new();

    loop {
        use tokio::time::{sleep_until, Duration, Instant};
        let st_time = Instant::now();
        match resp {
            QRCodeState::WaitingForScan => {
                py_try(|py| call_state(py, &handle_getter, "WaitingForScan", ()))?;
            }
            QRCodeState::WaitingForConfirm => {
                py_try(|py| call_state(py, &handle_getter, "WaitingForConfirm", ()))?;
            }
            QRCodeState::Canceled => {
                py_try(|py| call_state(py, &handle_getter, "Canceled", ()))?;
                resp = client.fetch_qrcode().await.py_res()?;
                continue;
            }
            QRCodeState::Timeout => {
                py_try(|py| call_state(py, &handle_getter, "Timeout", ()))?;
                resp = client.fetch_qrcode().await.py_res()?;
                continue;
            }
            QRCodeState::ImageFetch(ricq::QRCodeImageFetch {
                ref sig,
                ref image_data,
            }) => {
                image_sig = sig.clone();
                let qrcode_data = parse_qrcode(image_data)?;
                py_try(|py| call_state(py, &handle_getter, "DisplayQRCode", (qrcode_data,)))?;
            }
            QRCodeState::Confirmed(ricq::QRCodeConfirmed { uin, .. }) => {
                if uin == decl_uin {
                    py_try(|py| call_state(py, &handle_getter, "Success", (uin,)))?;
                    break;
                }
                py_try(|py| call_state(py, &handle_getter, "UINMismatch", (decl_uin, uin)))?;
                resp = client.fetch_qrcode().await.py_res()?;
                continue;
            }
        }
        sleep_until(st_time + Duration::from_secs_f64(interval)).await;
        resp = client.query_qrcode_result(&image_sig).await.py_res()?;
    }
    Ok(())
}

#[pyfunction]
pub fn qrcode_login<'py>(
    py: Python<'py>,
    uin: i64,
    protocol: &'py PyAny,
    store: &'py PyAny,
    event_callbacks: &'py PyList, // List[Callable[...]]
    login_callbacks: PyObject,    // QRCodeLoginCallbacks
) -> PyResult<&'py PyAny> {
    let (protocol, handler, device, token_rw) =
        parse_login_args(py, uin, protocol, store, event_callbacks)?;
    py_future(py, async move {
        let (client, alive) = prepare_client(device, protocol.clone(), handler).await?;
        if !token_rw.try_login(&client).await? {
            tracing::info!("正在使用二维码登录 {}", uin);
            let interval: f64 = py_try(|py| {
                login_callbacks
                    .as_ref(py)
                    .getattr("interval")?
                    .extract::<f64>()
            })?;
            let handle_getter: PyObject = py_try(|py| login_callbacks.getattr(py, "get_handle"))?;
            qrcode_login_process(&client, uin, handle_getter, interval).await?;
        }

        Ok(post_login(client, alive, token_rw).await?)
    })
}
