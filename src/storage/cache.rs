use std::sync::{RwLock, atomic::{AtomicI64, Ordering}};
use std::collections::HashMap;

use decky_api::StorePluginList;
use bytes::Bytes;
use chrono::Utc;

use super::IStorage;

struct Cached<T: Clone> {
    expiry: AtomicI64,
    value: RwLock<T>,
    ttl: i64
}

impl<T: Clone> Cached<T> {
    fn new(value: T, duration: i64) -> Self {
        Self {
            expiry: AtomicI64::new(Utc::now().timestamp() + duration),
            value: RwLock::new(value),
            ttl: duration,
        }
    }

    fn get<F: FnOnce() -> T>(&self, getter: F) -> T {
        let now = Utc::now().timestamp();
        let expiry = self.expiry.load(Ordering::Acquire);
        if expiry < now {
            // refresh required
            let new_value = getter();
            let new_expiry = now + self.ttl;
            let mut write_lock = self.value.write().expect("Failed to acquire cache write lock");
            self.expiry.store(new_expiry, Ordering::Release);
            *write_lock = new_value.clone();
            new_value
        } else {
            // cache is still good
            self.value.read().expect("Failed to acquire cache read lock").clone()
        }
    }

    fn refresh(&self, new_value: T) {
        let new_expiry = Utc::now().timestamp() + self.ttl;
        let mut write_lock = self.value.write().expect("Failed to acquire cache write lock");
        self.expiry.store(new_expiry, Ordering::Release);
        *write_lock = new_value;
    }
}

pub struct CachedStorage<S: AsRef<dyn IStorage> + Send + Sync> {
    fallback: S,
    plugins_cache: Cached<StorePluginList>,
    statistics_cache: Cached<HashMap<String, u64>>,
    artifacts_cache: Cached<HashMap<String, Bytes>>,
    images_cache: Cached<HashMap<String, Bytes>>,
}

impl<S: AsRef<dyn IStorage> + Send + Sync> CachedStorage<S> {
    pub fn new(duration: i64, inner: S) -> Self {
        Self {
            plugins_cache: Cached::new(inner.as_ref().plugins(), duration),
            statistics_cache: Cached::new(inner.as_ref().get_statistics(), duration),
            artifacts_cache: Cached::new(HashMap::new(), duration),
            images_cache: Cached::new(HashMap::new(), duration),
            fallback: inner,
        }
    }
}

impl<S: AsRef<dyn IStorage> + Send + Sync> IStorage for CachedStorage<S> {
    fn plugins(&self) -> StorePluginList {
        self.plugins_cache.get(|| self.fallback.as_ref().plugins())
    }

    fn get_artifact(&self, name: &str, version: &str, hash: &str) -> Result<bytes::Bytes, std::io::Error> {
        let mut cached = self.artifacts_cache.get(|| HashMap::new());
        if let Some(bytes) = cached.get(hash) {
            Ok(bytes.to_owned())
        } else {
            let new_artifact = self.fallback.as_ref().get_artifact(name, version, hash)?;
            cached.insert(hash.to_owned(), new_artifact.clone());
            self.artifacts_cache.refresh(cached);
            Ok(new_artifact)
        }
    }

    fn get_image(&self, name: &str) -> Result<bytes::Bytes, std::io::Error> {
        let mut cached = self.images_cache.get(|| HashMap::new());
        if let Some(bytes) = cached.get(name) {
            Ok(bytes.to_owned())
        } else {
            let new_image = self.fallback.as_ref().get_image(name)?;
            cached.insert(name.to_owned(), new_image.clone());
            self.images_cache.refresh(cached);
            Ok(new_image)
        }
    }

    fn get_statistics(&self) -> std::collections::HashMap<String, u64> {
        self.statistics_cache.get(|| self.fallback.as_ref().get_statistics())
    }
}
