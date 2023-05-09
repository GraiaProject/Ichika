use std::borrow::Borrow;
use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;
use std::time::{Duration, Instant};

use backon::{ExponentialBuilder, Retryable as _};
use lru_time_cache::LruCache;
use once_cell::sync::Lazy;
use ricq::{Client, RQError};
use tokio::sync::Mutex;

use super::structs::{FriendList, Group, Member};
use crate::exc::IckResult;

static CACHE_DURATION: Duration = Duration::from_secs(600);

static CACHE: Lazy<Mutex<HashMap<i64, Arc<Mutex<DetachedCache>>>>> = Lazy::new(Mutex::default);

static RETRY_BUILDER: Lazy<ExponentialBuilder> = Lazy::new(|| {
    ExponentialBuilder::default()
        .with_factor(1.5)
        .with_min_delay(Duration::from_secs(1))
        .with_max_delay(Duration::from_secs(5))
        .with_max_times(3)
});

#[repr(transparent)]
pub(crate) struct VarCache<T> {
    val: Option<(Instant, Arc<T>)>,
}

impl<T> Default for VarCache<T> {
    fn default() -> Self {
        Self { val: None }
    }
}

impl<T> VarCache<T> {
    fn get(&mut self) -> Option<Arc<T>> {
        if let Some((ref last_update, ref arc)) = self.val {
            if last_update.elapsed() <= CACHE_DURATION {
                return Some(arc.clone());
            }
            self.val = None;
        }
        None
    }

    fn set(&mut self, val: Arc<T>) -> Arc<T> {
        self.val = Some((Instant::now(), val.clone()));
        val
    }

    fn clear(&mut self) {
        self.val = None;
    }
}

#[repr(transparent)]
pub(crate) struct MapCache<K, V> {
    map: LruCache<K, Arc<V>>,
}

impl<K, V> Default for MapCache<K, V>
where
    K: Ord + Clone,
{
    fn default() -> Self {
        Self {
            map: LruCache::with_expiry_duration_and_capacity(CACHE_DURATION, 1024),
        }
    }
}

impl<K, V> MapCache<K, V>
where
    K: Ord + Clone,
{
    pub(crate) fn get<Q>(&mut self, key: &Q) -> Option<Arc<V>>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.map.get(key).cloned()
    }

    pub(crate) fn set(&mut self, key: K, val: Arc<V>) -> Arc<V> {
        self.map.insert(key, val.clone());
        val
    }

    pub(crate) fn remove<Q>(&mut self, key: &Q) -> Option<Arc<V>>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.map.remove(key)
    }
}

#[derive(Default)]
pub(crate) struct DetachedCache {
    pub(crate) friends: VarCache<FriendList>,
    pub(crate) groups: MapCache<i64, Group>,
    pub(crate) members: MapCache<(i64, i64), Member>,
}

impl VarCache<FriendList> {
    async fn fetch(&mut self, client: &Arc<Client>) -> IckResult<Arc<FriendList>> {
        if let Some(val) = self.get() {
            return Ok(val);
        }
        let fetch_closure =
            async move || -> IckResult<FriendList> { Ok(client.get_friend_list().await?.into()) };
        let val = Arc::new(fetch_closure.retry(&*RETRY_BUILDER).await?);
        Ok(self.set(val))
    }
}

impl MapCache<i64, Group> {
    async fn fetch(&mut self, client: &Arc<Client>, uin: i64) -> IckResult<Arc<Group>> {
        if let Some(val) = self.get(&uin) {
            return Ok(val);
        }
        let fetch_closure = async move || -> IckResult<Group> {
            Ok(client
                .get_group_info(uin)
                .await?
                .ok_or_else(|| RQError::EmptyField("group"))?
                .into())
        };
        let val = Arc::new(fetch_closure.retry(&*RETRY_BUILDER).await?);
        Ok(self.set(uin, val))
    }
}

impl MapCache<(i64, i64), Member> {
    async fn fetch(
        &mut self,
        client: &Arc<Client>,
        group_uin: i64,
        uin: i64,
    ) -> IckResult<Arc<Member>> {
        if let Some(val) = self.get(&(group_uin, uin)) {
            return Ok(val);
        }
        let fetch_closure = async move || -> IckResult<Member> {
            Ok(client.get_group_member_info(group_uin, uin).await?.into())
        };
        let val = Arc::new(fetch_closure.retry(&*RETRY_BUILDER).await?);
        Ok(self.set((group_uin, uin), val))
    }
}

pub struct ClientCache {
    pub(crate) client: Arc<Client>,
    pub(crate) detached: Arc<Mutex<DetachedCache>>,
}

impl ClientCache {
    pub async fn fetch_friend_list(&mut self) -> IckResult<Arc<FriendList>> {
        let mut guard = self.detached.lock().await;
        guard.friends.fetch(&self.client).await
    }

    pub async fn flush_friend_list(&mut self) {
        let mut guard = self.detached.lock().await;
        guard.friends.clear();
    }

    pub async fn fetch_group(&mut self, uin: i64) -> IckResult<Arc<Group>> {
        let mut guard = self.detached.lock().await;
        guard.groups.fetch(&self.client, uin).await
    }

    pub async fn flush_group(&mut self, uin: i64) {
        let mut guard = self.detached.lock().await;
        guard.groups.remove(&uin);
    }

    pub async fn fetch_member(&mut self, group_uin: i64, uin: i64) -> IckResult<Arc<Member>> {
        let mut guard = self.detached.lock().await;
        guard.members.fetch(&self.client, group_uin, uin).await
    }

    pub async fn flush_member(&mut self, group_uin: i64, uin: i64) {
        let mut guard = self.detached.lock().await;
        guard.members.remove(&(group_uin, uin));
    }
}

pub async fn cache(client: Arc<Client>) -> ClientCache {
    let uin = client.uin().await;
    let mut cache_guard = CACHE.lock().await;
    let detached = if let Some(detached) = cache_guard.get(&uin) {
        detached.clone()
    } else {
        let detached: Arc<Mutex<DetachedCache>> = Arc::default();
        cache_guard.insert(uin, detached.clone());
        detached
    };
    ClientCache { client, detached }
}
