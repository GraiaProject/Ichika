use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use async_trait::async_trait;
use ricq::client::{Client, Connector};
use tokio::net::TcpStream;
use tokio::task::JoinSet;
use tracing;

async fn tcp_connect_timeout(addr: SocketAddr, timeout: Duration) -> tokio::io::Result<TcpStream> {
    let conn = tokio::net::TcpStream::connect(addr);
    tokio::time::timeout(timeout, conn)
        .await
        .map_err(tokio::io::Error::from)
        .flatten()
}

/// Race the given address, call `join_set.join_next()` to get next fastest `(addr, conn)` pair.
async fn race_addrs(
    addrs: Vec<SocketAddr>,
    timeout: Duration,
) -> JoinSet<tokio::io::Result<(SocketAddr, TcpStream)>> {
    let mut join_set = JoinSet::new();
    for addr in addrs {
        join_set.spawn(async move {
            let a = addr;
            tcp_connect_timeout(addr, timeout).await.map(|s| {
                tracing::info!("地址 {} 连接成功", a);
                (a, s)
            })
        });
    }
    join_set
}

async fn tcp_connect_fastest(
    addrs: Vec<SocketAddr>,
    timeout: Duration,
) -> tokio::io::Result<TcpStream> {
    let mut join_set = race_addrs(addrs, timeout).await;
    while let Some(result) = join_set.join_next().await {
        if let Ok(Ok((_, stream))) = result {
            return Ok(stream);
        }
    }
    tracing::error!("无法连接至任何一个服务器");
    Err(tokio::io::Error::new(
        tokio::io::ErrorKind::NotConnected,
        "NotConnected",
    ))
}

pub struct IchikaConnector;

#[async_trait]
impl Connector<TcpStream> for IchikaConnector {
    async fn connect(&self, client: &Client) -> io::Result<TcpStream> {
        tcp_connect_fastest(client.get_address_list().await, Duration::from_secs(5)).await
    }
}
