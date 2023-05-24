mod connector;

use std::sync::Arc;

use connector::IchikaConnector;
use pyo3::exceptions::*;
use pyo3::prelude::*;
use pyo3::types::*;
use pyo3_asyncio::{into_future_with_locals, TaskLocals};
use pythonize::depythonize;
use ricq::client::{Client, Connector, NetworkStatus, Token};
use ricq::version::Version;
use ricq::{
    Device,
    LoginDeviceLocked,
    LoginNeedCaptcha,
    LoginResponse,
    LoginSuccess,
    LoginUnknownStatus,
    QRCodeState,
};
use tokio::task::JoinHandle;

use crate::events::PyHandler;
use crate::exc::{MapPyErr, RICQError};
use crate::utils::{partial, py_bytes, py_client_refs, py_future, py_try, py_use};
use crate::{exc, import_call, PyRet};

#[cfg(feature = "t544")]
pub(crate) mod t544 {
    use bytes::{BufMut, Bytes, BytesMut};
    use ricq::Client;
    use ricq_core::binary::BinaryWriter;
    use ricq_core::wtlogin::T544Provider;
    use t544_enc::t544_sign::sign;

    #[derive(Debug)]
    struct NativeT544Provider {
        uin: i64,
        guid: Bytes,
        version: Bytes,
    }

    impl NativeT544Provider {
        async fn new(client: &Client) -> Self {
            let uin = client.uin().await;
            let guid = client.engine.read().await.transport.sig.guid.clone();
            let version = client.version().await.sdk_version;

            Self {
                uin,
                guid,
                version: Bytes::copy_from_slice(version.as_bytes()),
            }
        }
    }

    impl T544Provider for NativeT544Provider {
        fn t544(&self, command: String) -> Bytes {
            let mut salt = BytesMut::new();
            let cmd = command.split("_").last().unwrap();
            let cmd = u32::from_str_radix(cmd, 16).unwrap();
            match cmd {
                2 | 7 => {
                    // T544v1
                    salt.put_u64(self.uin as u64);
                    salt.write_bytes_short(&self.guid[..]);
                    salt.write_bytes_short(&self.version[..]);
                    salt.put_u32(cmd);
                }
                _ => {
                    // T544v2
                    salt.put_u32(0);
                    salt.write_bytes_short(&self.guid[..]);
                    salt.write_bytes_short(&self.version[..]);
                    salt.put_u32(cmd);
                    salt.put_u32(0);
                }
            }
            let curr = std::time::UNIX_EPOCH.elapsed().unwrap().as_micros();
            let res = sign(curr as u64, &salt.freeze()[..]);
            Bytes::copy_from_slice(&res)
        }
    }

    pub(crate) async fn inject_t544(client: &mut Client) {
        client.engine.write().await.ex_provider.t544 =
            Some(Box::new(NativeT544Provider::new(client).await));
    }
}


async fn prepare_client(
    device: Device,
    app_ver: Version,
    handler: PyHandler,
) -> PyResult<(Arc<Client>, JoinHandle<()>)> {
    #[allow(unused_mut)]
    let mut client = Client::new(device, app_ver, handler);

    #[cfg(feature = "t544")]
    {
        t544::inject_t544(&mut client).await;
    }

    let client = Arc::new(client);
    let alive = tokio::spawn({
        let client = client.clone();
        // 连接最快的服务器
        let stream = IchikaConnector
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
    protocol: String,
    store: &'py PyAny,
    queues: &'py PyList,
) -> PyResult<(Version, PyHandler, Device, TokenRW, TaskLocals)> {
    let task_locals = TaskLocals::with_running_loop(py)?.copy_context(py)?; // Necessary since retrieving task locals at handling time is already insufficient
    let handler = PyHandler::new(queues.into_py(py), task_locals.clone(), uin);

    let get_token = partial(py).call1((store.getattr("get_token")?, uin, &protocol))?;
    let write_token = partial(py).call1((store.getattr("write_token")?, uin, &protocol))?;

    let device = store.getattr("get_device")?.call1((uin, &protocol))?; // JSON
    let device: Device = depythonize(device)
        .map_err(|e| exc::LoginError::new_err(format!("无法解析传入的设备信息: {e:?}")))?;

    Ok((
        ricq::version::get_version(
            ricq::Protocol::try_from(protocol.as_ref())
                .map_err(|_| PyValueError::new_err(format!("无法找到协议 {protocol}")))?,
        ),
        handler,
        device,
        TokenRW {
            get_token: get_token.into_py(py),
            write_token: write_token.into_py(py),
        },
        task_locals,
    ))
}

async fn invoke_cb(
    locals: &TaskLocals,
    getter: &PyObject,
    name: &str,
    args: impl IntoPy<Py<PyTuple>>,
) -> PyRet {
    let (obj, is_none) = py_try(|py| {
        let handler = getter.as_ref(py).call1((name,))?;
        if handler.is_none() {
            return Ok((py.None(), true)); // return None
        }
        Ok((handler.call1(args)?.into_py(py), false))
    })?;
    if is_none {
        return Ok(obj);
    }
    py_use(|py| into_future_with_locals(locals, obj.as_ref(py)))?.await
}

pub async fn reconnect(
    client: &Arc<Client>,
    token_rw: &TokenRW,
) -> PyResult<Option<JoinHandle<()>>> {
    use std::time::Duration;

    use backon::{ExponentialBuilder, Retryable as _};

    let uin = client.uin().await;

    let retry_builder = ExponentialBuilder::default()
        .with_factor(1.2)
        .with_min_delay(Duration::from_secs(3))
        .with_max_delay(Duration::from_secs(60))
        .with_max_times(usize::MAX);
    let retry_closure = || async {
        // 如果不是网络原因掉线，不重连（服务端强制下线/被踢下线/用户手动停止）
        if client.get_status() != (NetworkStatus::NetworkOffline as u8) {
            tracing::warn!("账号 {} 因非网络原因下线，不再重连", uin);
            return Ok(None);
        }
        client.stop(NetworkStatus::NetworkOffline);

        let alive = tokio::spawn({
            let client = client.clone();
            // 连接最快的服务器
            let stream = IchikaConnector.connect(&client).await?;

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
            return Err(ricq_core::error::RQError::Network).py_res();
        }

        after_login(client).await?;

        tracing::info!("客户端重连成功");
        Ok(Some(alive))
    };
    let retry_closure = || async move { retry_closure().await.map_err(|e| (uin, e)) };
    retry_closure
        .retry(&retry_builder)
        .notify(|e, dur: Duration| {
            tracing::error!(
                "客户端 {} 重连失败，原因：{}，将在 {:.2} 秒后重试",
                e.0,
                e.1,
                dur.as_secs_f64()
            );
        })
        .await
        .map_err(|e| e.1)
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
    locals: &TaskLocals,
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
    let verify_url = data.verify_url.as_ref().map_or_else(
        || Err(exc::RICQError::new_err("无法获取验证地址")),
        |url| Ok(url.clone()),
    )?;
    tracing::info!("{:?}", data.clone());
    if let Some(sms_phone) = sms_phone
        && sms
        && let Ok(rsp) = client.request_sms().await {
            if !matches!(rsp, LoginResponse::DeviceLocked(_)) {
                return Ok(rsp);
            }

            let res = invoke_cb(locals, &handle_getter, "RequestSMS", (message, sms_phone)).await?;
            let sms_code = py_try(|py| res.extract::<Option<String>>(py))?;

            if let Some(sms_code) = sms_code {
                return client
                .submit_sms_code(&sms_code)
                .await
                .py_res();
            }
        }
    invoke_cb(
        locals,
        &handle_getter,
        "DeviceLocked",
        (message, verify_url),
    )
    .await?;
    make_password_login_req(uin, client, credential)
        .await
        .py_res()
}

async fn password_login_process(
    locals: &TaskLocals,
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
                invoke_cb(locals, &handle_getter, "Success", ()).await?;
                break;
            }
            LoginResponse::DeviceLocked(data) => {
                resp = handle_device_lock(
                    locals,
                    &data,
                    uin,
                    client,
                    &credential,
                    handle_getter.clone(),
                    sms,
                )
                .await?;
            }
            LoginResponse::NeedCaptcha(LoginNeedCaptcha { ref verify_url, .. }) => {
                let verify_url = verify_url.as_ref().map_or_else(
                    || Err(exc::RICQError::new_err("无法获取验证地址")),
                    |url| Ok(url.clone()),
                )?;
                let ticket =
                    invoke_cb(locals, &handle_getter, "NeedCaptcha", (verify_url,)).await?;
                let ticket = py_try(|py| ticket.extract::<String>(py))?;
                resp = client.submit_ticket(&ticket).await.py_res()?;
            }
            LoginResponse::DeviceLockLogin { .. } => {
                invoke_cb(locals, &handle_getter, "DeviceLockLogin", ()).await?;
                resp = client.device_lock_login().await.py_res()?;
            }
            LoginResponse::AccountFrozen => {
                invoke_cb(locals, &handle_getter, "AccountFrozen", ()).await?;
                break;
            }
            LoginResponse::TooManySMSRequest => {
                invoke_cb(locals, &handle_getter, "TooManySMSRequest", ()).await?;
                break;
            }
            LoginResponse::UnknownStatus(LoginUnknownStatus {
                ref status,
                ref message,
                ..
            }) => {
                let (status, message) = (*status, message.clone());
                invoke_cb(locals, &handle_getter, "UnknownStatus", (message, status)).await?;
                break;
            }
        }
    }

    Ok(())
}


async fn after_login(client: &Arc<Client>) -> PyResult<()> {
    client
        .register_client()
        .await
        .map_err(|e| exc::RICQError::new_err(format!("注册客户端失败: {e:?}")))?;

    if !client
        .heartbeat_enabled
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        let client = client.clone();
        tokio::spawn(async move {
            client.do_heartbeat().await;
        });
    }

    client
        .refresh_status()
        .await
        .map_err(|e| exc::RICQError::new_err(format!("刷新状态失败: {e:?}")))?;

    Ok(())
}

async fn post_login(client: Arc<Client>, alive: JoinHandle<()>, token_rw: TokenRW) -> PyRet {
    after_login(&client).await?;

    token_rw.set(&client).await?;
    let uin = client.uin().await;
    let init = crate::client::ClientInitializer {
        uin,
        client,
        alive: Arc::new(std::sync::Mutex::new(Some(alive))),
        token_rw,
    };
    py_try(|py| {
        let client = import_call!(py, "ichika.client" => "Client" => init)?.into_py(py);
        py_client_refs(py).set_item(uin, client.clone_ref(py))?;
        Ok(client)
    })
}

#[pyfunction]
#[allow(clippy::too_many_arguments, reason = "Required for Python binding")]
pub fn password_login<'py>(
    py: Python<'py>,
    uin: i64,
    credential: PasswordCredential,
    use_sms: bool,
    protocol: String,
    store: &'py PyAny,
    queues: &'py PyList,       // List[asyncio.Queue[Event]]
    login_callbacks: PyObject, // PasswordLoginCallbacks
) -> PyResult<&'py PyAny> {
    let (protocol, handler, device, token_rw, locals) =
        parse_login_args(py, uin, protocol, store, queues)?;
    py_future(py, async move {
        let (client, alive) = prepare_client(device, protocol.clone(), handler).await?;
        if !token_rw.try_login(&client).await? {
            tracing::info!("正在使用密码登录 {}", uin);
            let handle_getter: PyObject = py_try(|py| login_callbacks.getattr(py, "get_handle"))?;
            password_login_process(&locals, &client, uin, credential, use_sms, handle_getter)
                .await?;
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
    locals: &TaskLocals,
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
                invoke_cb(locals, &handle_getter, "WaitingForScan", ()).await?;
            }
            QRCodeState::WaitingForConfirm => {
                invoke_cb(locals, &handle_getter, "WaitingForConfirm", ()).await?;
            }
            QRCodeState::Canceled => {
                invoke_cb(locals, &handle_getter, "Canceled", ()).await?;
                resp = client.fetch_qrcode().await.py_res()?;
                continue;
            }
            QRCodeState::Timeout => {
                invoke_cb(locals, &handle_getter, "Timeout", ()).await?;
                resp = client.fetch_qrcode().await.py_res()?;
                continue;
            }
            QRCodeState::ImageFetch(ricq::QRCodeImageFetch {
                ref sig,
                ref image_data,
            }) => {
                image_sig = sig.clone();
                let qrcode_data = parse_qrcode(image_data)?;
                invoke_cb(locals, &handle_getter, "DisplayQRCode", (qrcode_data,)).await?;
            }
            QRCodeState::Confirmed(ricq::QRCodeConfirmed {
                uin,
                ref tmp_pwd,
                ref tmp_no_pic_sig,
                ref tgt_qr,
                ..
            }) => {
                if uin == decl_uin {
                    let mut login_resp = client
                        .qrcode_login(tmp_pwd, tmp_no_pic_sig, tgt_qr)
                        .await
                        .py_res()?;
                    if matches!(login_resp, LoginResponse::DeviceLockLogin(_)) {
                        tracing::info!("账号 {} 尝试设备锁登录", uin);
                        login_resp = client.device_lock_login().await.py_res()?;
                    }
                    if matches!(login_resp, LoginResponse::Success(_)) {
                        invoke_cb(locals, &handle_getter, "Success", (uin,)).await?;
                        break;
                    }
                    Err(RICQError::new_err(format!("登录失败: {login_resp:?}")))?;
                }
                invoke_cb(locals, &handle_getter, "UINMismatch", (decl_uin, uin)).await?;
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
    protocol: String,
    store: &'py PyAny,
    queues: &'py PyList,       // List[asyncio.Queue[Event]]
    login_callbacks: PyObject, // QRCodeLoginCallbacks
) -> PyResult<&'py PyAny> {
    let (protocol, handler, device, token_rw, locals) =
        parse_login_args(py, uin, protocol, store, queues)?;
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
            qrcode_login_process(&locals, &client, uin, handle_getter, interval).await?;
        }

        Ok(post_login(client, alive, token_rw).await?)
    })
}
