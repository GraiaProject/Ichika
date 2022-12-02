use std::{ops::DerefMut, sync::Arc, time::Duration};

use anyhow::Result;
use futures_util::Future;
use tokio::{sync::Mutex, time::Instant};

use super::friend_list::FriendList;

pub struct ClientImpl {
    inner: Arc<ricq::Client>,
    friend_list: Arc<Cached<FriendList>>,
}

impl ClientImpl {
    pub fn new(inner: Arc<ricq::Client>) -> Self {
        Self {
            inner,
            friend_list: Arc::new(Cached::new(Duration::from_secs(3600))),
        }
    }

    pub fn inner(&self) -> &Arc<ricq::Client> {
        &self.inner
    }

    pub async fn get_friend_list(self: Arc<Self>) -> Result<FriendList> {
        let friend_list = self.inner.get_friend_list().await?;
        let friend_list = FriendList {
            client: self.clone(),
            friends: friend_list.friends,
            friend_groups: friend_list.friend_groups,
            total_count: friend_list.total_count,
            online_count: friend_list.online_friend_count,
        };
        Ok(friend_list)
    }

    pub async fn get_friend_list_cached(self: Arc<Self>) -> Result<FriendList> {
        self.friend_list.get(self.clone()).await
    }
}

/// 缓存。
pub struct Cached<T: Cacheable> {
    last_updated_value: Mutex<(Option<T>, Instant)>,
    duration: Duration,
}

/// 可缓存的值。
// #[async_trait]
pub trait Cacheable: Clone {
    type FetchFuture: Future<Output = Result<Self>>;
    /// 从远程获取值。
    fn fetch(client: Arc<ClientImpl>) -> Self::FetchFuture;
}

impl<T: Cacheable> Cached<T> {
    /// 创建一个新的缓存。
    ///
    /// # Arguments
    /// * `duration` - 缓存时长。
    fn new(duration: Duration) -> Self {
        Self {
            last_updated_value: Mutex::new((None, Instant::now())),
            duration,
        }
    }

    /// 获取缓存，如果缓存过期或不存在则更新缓存。
    async fn get(&self, client: Arc<ClientImpl>) -> Result<T> {
        let mut locked = self.last_updated_value.lock().await;
        let (cache, last_update) = locked.deref_mut();
        if cache.is_none() || last_update.elapsed() > self.duration {
            let value = T::fetch(client).await?;
            *cache = Some(value.clone());
            *last_update = Instant::now();
            return Ok(value);
        }
        Ok(cache.clone().unwrap())
    }
}
