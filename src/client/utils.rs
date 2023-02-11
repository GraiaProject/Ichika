use std::ops::DerefMut;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use futures_util::Future;
use ricq::client::Client;
use tokio::sync::Mutex;
use tokio::time::Instant;

pub struct CacheField<T: CacheTarget> {
    last_updated_value: Mutex<(Option<T>, Instant)>,
    duration: Duration,
}

pub trait CacheTarget: Clone {
    type FetchFuture: Future<Output = Result<Self>>;
    /// 从远程获取值。
    fn fetch(client: Arc<Client>) -> Self::FetchFuture;
}

impl<T: CacheTarget> CacheField<T> {
    pub fn new(duration: Duration) -> Self {
        Self {
            last_updated_value: Mutex::new((None, Instant::now())),
            duration,
        }
    }

    pub async fn clear(&self) {
        let mut locked = self.last_updated_value.lock().await;
        let (cache, last_update) = locked.deref_mut();
        *cache = None;
        *last_update = Instant::now();
    }

    pub async fn get(&self, client: Arc<Client>) -> Result<T> {
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
