use pyo3::prelude::*;
use ricq::client::event as rce;
use ricq::handler::QEvent;
use ricq::RQError;

use super::structs::{FriendInfo, GroupInfo, MemberInfo, MessageSource};
use super::{FriendMessage, GroupMessage, LoginEvent, TempMessage, UnknownEvent};
use crate::exc::MapPyErr;
use crate::message::convert::deserialize;
use crate::utils::{py_try, py_use};
use crate::PyRet;

pub async fn convert(event: QEvent) -> PyRet {
    match event {
        QEvent::Login(event) => handle_login(event).await,
        QEvent::GroupMessage(event) => handle_group_message(event).await,
        QEvent::FriendMessage(event) => handle_friend_message(event).await,
        QEvent::GroupTempMessage(event) => handle_temp_message(event).await,
        unknown => obj(|_| UnknownEvent { inner: unknown }),
    }
}

fn obj<F, R, T>(f: F) -> PyResult<T>
where
    F: for<'py> FnOnce(Python<'py>) -> R,
    R: IntoPy<T>,
{
    py_use(|py| Ok(f(py).into_py(py)))
}

async fn handle_login(uin: i64) -> PyRet {
    obj(|py| LoginEvent { uin }.into_py(py))
}

async fn handle_group_message(event: rce::GroupMessageEvent) -> PyRet {
    let msg = event.inner;
    let client = event.client;
    let sender_info = client
        .get_group_member_info(msg.group_code, msg.from_uin)
        .await
        .py_res()?;
    let content = py_try(|py| deserialize(py, msg.elements))?;
    obj(|py| GroupMessage {
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time),
        content,
        sender: MemberInfo {
            uin: msg.from_uin,
            name: sender_info.card_name,
            group: GroupInfo {
                uin: msg.group_code,
                name: msg.group_name,
            },
            permission: sender_info.permission as u8,
        },
    })
}

async fn handle_friend_message(event: rce::FriendMessageEvent) -> PyRet {
    let msg = event.inner;
    let content = py_try(|py| deserialize(py, msg.elements))?;
    obj(|py| FriendMessage {
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time),
        content,
        sender: FriendInfo {
            uin: msg.from_uin,
            nickname: msg.from_nick,
        },
    })
}

async fn handle_temp_message(event: rce::GroupTempMessageEvent) -> PyRet {
    let msg = event.inner;
    let content = py_try(|py| deserialize(py, msg.elements))?;
    let client = event.client;
    let group_info = client
        .get_group_info(msg.group_code)
        .await
        .transpose()
        .unwrap_or_else(|| Err(RQError::UnsuccessfulRetCode(-1)))
        .py_res()?;
    let sender_info = client
        .get_group_member_info(msg.group_code, msg.from_uin)
        .await
        .py_res()?;
    obj(|py| TempMessage {
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time),
        content,
        sender: MemberInfo {
            uin: msg.from_uin,
            name: sender_info.card_name,
            group: GroupInfo {
                uin: group_info.code,
                name: group_info.name,
            },
            permission: sender_info.permission as u8,
        },
    })
}
