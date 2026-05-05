use std::sync::{Mutex, Condvar, Arc, MutexGuard};
use std::collections::hash_map::{Entry};
use std::hash::Hash;
use std::mem::replace;
use web_time::{Instant, Duration};
use super::{ValueSize, CacheControl, global::GlobalCache};
use async_trait::async_trait;
use rustc_hash::FxHashMap as HashMap;

struct Computed<V> {
    value: V,
    time: f64,
    size: usize,
    last_used: Instant,
}
enum Value<V> {
    InProcess(Arc<Condvar>),
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
#[derive(Default)]
pub struct SyncCache<K, V> {
    name: Option<String>,
    inner: Mutex<CacheInner<K, V>>,
}
#[derive(Default)]
struct CacheInner<K, V> {
    entries: HashMap<K, Value<V>>,
}

impl<K, V> SyncCache<K, V> 
    where K: Hash + Eq + Clone + Send + 'static, V: Clone + ValueSize + Send + 'static,
{
    pub fn new() -> Arc<Self> {
        Self::with_name(None)
    }
    pub fn with_name(name: Option<String>) -> Arc<Self> {
        let cache = Arc::new(SyncCache {
            name,
            inner: Mutex::new(CacheInner {
                entries: HashMap::default(),
            })
        });
        GlobalCache::register(Arc::downgrade(&cache));
        cache
    }
    pub fn get(&self, key: K, compute: impl FnOnce(&K) -> V) -> V {
        let mut guard = self.inner.lock().unwrap();
        match guard.entries.entry(key) {
            Entry::Occupied(e) => match e.get() {
                &Value::Computed(ref v) => return v.value.clone(),
                &Value::InProcess(ref condvar) => {
                    let key = e.key().clone();
                    let condvar = condvar.clone();
                    return Self::poll(key, guard, condvar);
                }
            }
            Entry::Vacant(e) => {
                let key = e.key().clone();
                let condvar = Arc::new(Condvar::new());
                e.insert(Value::InProcess(condvar));
                drop(guard);

                let start = Instant::now();
                let value = compute(&key);
                let size = value.size();
                let duration = start.elapsed();
                let value2 = value.clone();
                let time = duration.as_secs_f64() + 0.000001;
                let last_used = Instant::now();
                let mut guard = self.inner.lock().unwrap();

                let c = Computed {
                    value,
                    size,
                    time,
                    last_used,
                };
                let slot = guard.entries.get_mut(&key).unwrap();
                let slot = replace(slot, Value::Computed(c));
                match slot {
                    Value::InProcess(ref condvar) => condvar.notify_all(),
                    _ => unreachable!()
                }
                return value2;
            }
        }
    }
    pub fn entries(arc: Arc<Self>) -> impl Iterator<Item=(K, V)> {
        let cache = Arc::try_unwrap(arc).ok().unwrap();
            cache.inner.into_inner().unwrap()
            .entries
            .into_iter()
            .map(|(k, v)| (k, v.unwrap()))
    }
    pub fn remove(&self, key: &K) {
        self.inner.lock().unwrap().entries.remove(key);
    }

    pub fn clear(&self) {
        self.inner.lock().unwrap().entries.clear()
    }
    fn poll(key: K, mut guard: MutexGuard<CacheInner<K, V>>, condvar: Arc<Condvar>) -> V {
        let last_used = Instant::now();
        loop {
            guard = condvar.wait(guard).unwrap();
            let inner = &mut *guard;
            if let &mut Value::Computed(ref mut v) = inner.entries.get_mut(&key).unwrap() {
                v.last_used = last_used;
                return v.value.clone();
            }
        }
    }
}

#[async_trait]
impl<K, V> CacheControl for SyncCache<K, V>
    where K: Eq + Hash + Send + 'static, V: Clone + ValueSize + Send + 'static
{
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    async fn clean(&self, threshold: f64, time_scale: f64) -> (usize, f64) {
        self.inner.lock().unwrap().clean(threshold, time_scale)
    }
}

impl<K, V> CacheInner<K, V> {
    fn clean(&mut self, threshold: f64, time_scale: f64) -> (usize, f64) {
        let now = Instant::now() + Duration::from_secs_f64(time_scale);
        let mut size_sum = 0;

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
