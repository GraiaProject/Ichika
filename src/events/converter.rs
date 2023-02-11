use pyo3::prelude::*;
use ricq::handler::QEvent;

use super::structs::{FriendInfo, GroupInfo, MemberInfo, MessageSource};
use super::{FriendMessage, GroupMessage, LoginEvent, UnknownEvent};
use crate::message::convert::deserialize;
use crate::RICQError;

macro_rules! converter {
    ($($event_type: ident => [$event_cap: ident] $body: block);*) => {
        pub async fn convert(event: QEvent) -> PyResult<PyObject> {
            match event {
                $(QEvent::$event_type(e) => {let $event_cap = e; $body },)*
                unknown => Ok(Python::with_gil(|py|{UnknownEvent { inner: unknown }.into_py(py)}))
            }
        }
    };
}

converter!(
    Login => [uin] {
        Ok(Python::with_gil(|py| LoginEvent {uin}.into_py(py)))
    };
    GroupMessage => [event] {
    let msg = event.inner;
    let client = event.client;
    let sender_info = client
    .get_group_member_info(msg.group_code, msg.from_uin)
    .await
    .map_err(RICQError)?;
    Python::with_gil(|py| {
    Ok(GroupMessage {
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time),
        content: deserialize(py, msg.elements)?,
        sender: MemberInfo {
            uin: msg.from_uin,
            name: sender_info.card_name,
            group: GroupInfo {
                uin: msg.group_code,
                name: msg.group_name,
            },
            permission: sender_info.permission as u8,
        },
    }
    .into_py(py))})
};    FriendMessage => [event] {
    Python::with_gil(|py| {
        let msg = event.inner;
    Ok(FriendMessage {source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time), content: deserialize(py, msg.elements)?,
    sender: FriendInfo {uin: msg.from_uin, nickname: msg.from_nick}}.into_py(py))})
});
