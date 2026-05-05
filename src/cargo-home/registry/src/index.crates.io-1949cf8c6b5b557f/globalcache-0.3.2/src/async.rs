use std::collections::hash_map::{HashMap, Entry};
use std::hash::Hash;
use std::mem::replace;
use std::sync::Arc;
use tokio::sync::{Notify, Mutex};
use tokio::time::{Instant, Duration};
use super::{ValueSize, CacheControl, global::GlobalCache};
use std::future::{Future, ready};
use async_trait::async_trait;

struct Computed<V> {
    value: V,
    size: usize,
    time: f64,
    last_used: Instant,
}
enum Value<V> {
    InProcess(Arc<Notify>),
    Computed(Computed<V>),
}
impl<V> Value<V> {
    fn unwrap(self) -> V {
        match self {
            Value::Computed(v) => v.value,
            _ => unreachable!()
        }
    }
}

pub struct AsyncCache<K, V> {
    name: Option<String>,
    inner: Mutex<CacheInner<K, V>>,
}
struct CacheInner<K, V> {
    entries: HashMap<K, Value<V>>,
}

impl<K, V> AsyncCache<K, V> 
    where K: Clone + Hash + Eq + Send + 'static, V: Clone + ValueSize + Send + 'static,
{
    pub fn new() -> Arc<Self> {
        Self::with_name(None)
    }
    pub fn with_name(name: Option<String>) -> Arc<Self> {
        let cache = Arc::new(AsyncCache {
            name: name.into(),
            inner: Mutex::new(CacheInner {
                entries: HashMap::new(),
            })
        });
        GlobalCache::register(Arc::downgrade(&cache));
        cache
    }
    pub fn entries(self) -> impl Iterator<Item=(K, V)> {
        self.inner.into_inner()
            .entries
            .into_iter()
            .map(|(k, v)| (k, v.unwrap()))
    }
    pub async fn get(&self, key: K, compute: impl FnOnce(&K) -> V) -> V {
        self.get_async(key, |key| ready(compute(key))).await
    }
    pub async fn get_async<F, C>(&self, key: K, compute: C) -> V
    where
        F: Future<Output=V>,
        C: FnOnce(&K) -> F
    {
        let mut guard = self.inner.lock().await;
        let key2 = key.clone();
        match guard.entries.entry(key) {
            Entry::Occupied(mut e) => match e.get_mut() {
                &mut Value::Computed(ref mut v) => {
                    v.last_used = Instant::now();
                    return v.value.clone()
                }
                &mut Value::InProcess(ref condvar) => {
                    let condvar = condvar.clone();
                    drop(guard);
                    return self.poll(key2, condvar).await;
                }
            }
            Entry::Vacant(e) => {
                let key = e.key().clone();
                let notify = Arc::new(Notify::new());
                e.insert(Value::InProcess(notify));
                drop(guard);

                let start = Instant::now();
                let value = compute(&key).await;
                let size = value.size();
                let duration = start.elapsed();
                let value2 = value.clone();
                let time = duration.as_secs_f64() + 0.000001;
                let mut guard = self.inner.lock().await;

                let c = Computed {
                    value,
                    size,
                    time,
                    last_used: start
                };
                let slot = guard.entries.get_mut(&key).unwrap();
                let slot = replace(slot, Value::Computed(c));
                match slot {
                    Value::InProcess(ref notify) => notify.notify_waiters(),
                    _ => unreachable!()
                }
                return value2;
            }
        }
    }
    async fn poll(&self, key: K, notify: Arc<Notify>) -> V {
        loop {
            notify.notified().await;
            let mut guard = self.inner.lock().await;
            let inner = &mut *guard;
            if let &mut Value::Computed(ref mut v) = inner.entries.get_mut(&key).unwrap() {
                v.last_used = Instant::now();
                return v.value.clone();
            }
        }
    }
    pub async fn remove(&self, key: &K) {
        self.inner.lock().await.entries.remove(key);
    }
    pub async fn clear(&self) {
        self.inner.lock().await.entries.clear()
    }
}

#[async_trait]
impl<K, V> CacheControl for AsyncCache<K, V>
    where K: Eq + Hash + Send + 'static, V: Clone + ValueSize + Send + 'static
{
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    async fn clean(&self, threshold: f64, time_scale: f64) -> (usize, f64) {
        self.inner.lock().await.clean(threshold, time_scale)
    }
}

impl<K, V> CacheInner<K, V> {
    fn clean(&mut self, threshold: f64, time_scale: f64) -> (usize, f64) {
        let mut size_sum = 0;
        let now = Instant::now() + Duration::from_secs_f64(time_scale);

        let mut time_sum = 0.0;
        self.entries.retain(|_, value| {
            match value {
                Value::Computed(ref entry) => {
                    let elapsed = now.duration_since(entry.last_used);
                    let value = entry.time / (entry.size as f64 * elapsed.as_secs_f64());
                    if value > threshold {
                        time_sum += entry.time;
                        size_sum += entry.size;
                        true
                    } else {
                        false
                    }
                }
                Value::InProcess(_) => true
            }
        });
        (size_sum, time_sum)
    }
}
