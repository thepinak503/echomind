use tracing::debug;
use tokio::sync::mpsc;
use std::sync::Weak;
use once_cell::sync::OnceCell;
use super::CacheControl;
use std::future::Future;

pub struct GlobalCache {
    register: mpsc::UnboundedSender<Weak<dyn CacheControl>>,
}
static GLOBAL: OnceCell<GlobalCache> = OnceCell::new();

pub fn global_init(config: CacheConfig) {
    tokio::spawn(global_cleaner(config));
}

pub struct CacheConfig {
    //// memory limit in bytes
    pub memory_limit: usize,

    /// time scale in seconds
    pub time_scale: f64,
}

pub fn global_cleaner(config: CacheConfig) -> impl Future<Output=()> {
    use tokio::time::{Duration, sleep};

    let (tx, mut rx) = mpsc::unbounded_channel();
    GlobalCache { register: tx }.make_global();

    async move {
        let mut slots = vec![];
        let mut last_threshold = 0.0;
        let mut new_slots = vec![];
        loop {
            while let Ok(new) = rx.try_recv() {
                slots.push(new);
            }

            let mut total_size = 0;
            let mut total_time = 0.;
            for slot in slots.drain(..) {
                if let Some(cache) = Weak::upgrade(&slot) {
                    let (size, time_sum) = cache.clean(last_threshold, config.time_scale).await;
                    total_size += size;
                    total_time += time_sum;
                    new_slots.push(slot); // keep it
                }
            }
            std::mem::swap(&mut slots, &mut new_slots);

            if total_size > 0 {
                let value = total_time / (total_size as f64 * config.time_scale);
                let scale = total_size as f64 / config.memory_limit as f64;
                last_threshold = value * scale;
            } else {
                last_threshold = 0.0;
            }

            debug!("{} seconds cached in {} bytes", total_time, total_size);
            //info!("new threshold: {}", last_threshold);

            sleep(Duration::from_secs(1)).await;
        }
    }
}

impl GlobalCache {
    pub fn global() -> &'static Self {
        GLOBAL.get().unwrap()
    }
    pub fn make_global(self) {
        GLOBAL.set(self).ok().expect("");
    }
    pub fn register(cache: Weak<impl CacheControl>) {
        if let Some(global) = GLOBAL.get() {
            global.register.send(cache as _).ok().unwrap();
        }
    }
}
