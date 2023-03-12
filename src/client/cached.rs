use std::borrow::Borrow;
use std::collections::HashMap;
use std::default::Default;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use ricq::{Client, RQError, RQResult};
use tokio::sync::Mutex;

use super::friend::FriendList;
use super::group::{Group, Member};

static CACHE_DURATION: Duration = Duration::from_secs(600);

static CACHE: Lazy<Mutex<HashMap<i64, Arc<Mutex<DetachedCache>>>>> = Lazy::new(Mutex::default);

#[repr(transparent)]
struct VarCache<T> {
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
struct MapCache<K, V> {
    map: HashMap<K, (Instant, Arc<V>)>,
}

impl<K, V> Default for MapCache<K, V> {
    fn default() -> Self {
        Self {
            map: HashMap::default(),
        }
    }
}

impl<K, V> MapCache<K, V>
where
    K: Eq + Hash,
{
    fn get<Q>(&mut self, key: &Q) -> Option<Arc<V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        if let Some((ref last_upd, ref arc)) = self.map.get(key) {
            if last_upd.elapsed() <= CACHE_DURATION {
                return Some(arc.clone());
            }
            self.map.remove(key);
        }
        None
    }

    fn set(&mut self, key: K, val: Arc<V>) -> Arc<V> {
        self.map.insert(key, (Instant::now(), val.clone()));
        val
    }

    fn remove<Q>(&mut self, key: &Q) -> Option<Arc<V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.map.remove(key).map(|v| v.1)
    }
}

#[derive(Default)]
struct DetachedCache {
    friends: VarCache<FriendList>,
    groups: MapCache<i64, Group>,
    members: MapCache<(i64, i64), Member>,
}

impl VarCache<FriendList> {
    async fn fetch(&mut self, client: &Arc<Client>) -> RQResult<Arc<FriendList>> {
        if let Some(val) = self.get() {
            return Ok(val);
        }
        let val = Arc::new(client.get_friend_list().await?.into());
        Ok(self.set(val))
    }
}

impl MapCache<i64, Group> {
    async fn fetch(&mut self, client: &Arc<Client>, uin: i64) -> RQResult<Arc<Group>> {
        if let Some(val) = self.get(&uin) {
            return Ok(val);
        }
        let val = Arc::new(
            client
                .get_group_info(uin)
                .await?
                .ok_or_else(|| RQError::EmptyField("group"))?
                .into(),
        );
        Ok(self.set(uin, val))
    }
}

impl MapCache<(i64, i64), Member> {
    async fn fetch(
        &mut self,
        client: &Arc<Client>,
        group_uin: i64,
        uin: i64,
    ) -> RQResult<Arc<Member>> {
        if let Some(val) = self.get(&(group_uin, uin)) {
            return Ok(val);
        }
        let val = Arc::new(client.get_group_member_info(group_uin, uin).await?.into());
        Ok(self.set((group_uin, uin), val))
    }
}

pub struct ClientCache {
    client: Arc<Client>,
    detached: Arc<Mutex<DetachedCache>>,
}

impl ClientCache {
    pub async fn fetch_friend_list(&mut self) -> RQResult<Arc<FriendList>> {
        let mut guard = self.detached.lock().await;
        guard.friends.fetch(&self.client).await
    }

    pub async fn flush_friend_list(&mut self) {
        let mut guard = self.detached.lock().await;
        guard.friends.clear();
    }

    pub async fn fetch_group(&mut self, uin: i64) -> RQResult<Arc<Group>> {
        let mut guard = self.detached.lock().await;
        guard.groups.fetch(&self.client, uin).await
    }

    pub async fn flush_group(&mut self, uin: i64) {
        let mut guard = self.detached.lock().await;
        guard.groups.remove(&uin);
    }

    pub async fn fetch_member(&mut self, group_uin: i64, uin: i64) -> RQResult<Arc<Member>> {
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
    let detached = match cache_guard.get(&uin) {
        Some(detached) => detached.clone(),
        None => {
            let detached: Arc<Mutex<DetachedCache>> = Arc::default();
            cache_guard.insert(uin, detached.clone());
            detached
        }
    };
    ClientCache { client, detached }
}
