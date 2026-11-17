//! Tiny TTL cache used to avoid re-probing the whole host environment on every
//! frontend refresh. Entries are invalidated explicitly after a mutation that is
//! known to change the result (install / uninstall / version switch).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct TtlCache<T> {
    ttl: Duration,
    entries: Mutex<Option<HashMap<String, (Instant, T)>>>,
    /// Bumped by `clear`, so a probe that started earlier cannot store its result.
    epoch: AtomicU64,
}

impl<T: Clone> TtlCache<T> {
    pub const fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: Mutex::new(None),
            epoch: AtomicU64::new(0),
        }
    }

    /// Read this before computing a value and hand it to `put`.
    pub fn epoch(&self) -> u64 {
        self.epoch.load(Ordering::Acquire)
    }

    /// Returns the cached value only while it is younger than the TTL.
    pub fn get(&self, key: &str) -> Option<T> {
        let guard = self.entries.lock().ok()?;
        let map = guard.as_ref()?;
        let (stored_at, value) = map.get(key)?;
        (stored_at.elapsed() < self.ttl).then(|| value.clone())
    }

    /// Stores `value` unless the cache was cleared after `epoch` was read:
    /// such a value was computed from the state before the mutation.
    pub fn put(&self, key: &str, value: T, epoch: u64) {
        if let Ok(mut guard) = self.entries.lock() {
            if self.epoch() != epoch {
                return;
            }
            guard
                .get_or_insert_with(HashMap::new)
                .insert(key.to_string(), (Instant::now(), value));
        }
    }

    /// Drop one entry so the next `get` recomputes it.
    pub fn invalidate(&self, key: &str) {
        if let Ok(mut guard) = self.entries.lock() {
            if let Some(map) = guard.as_mut() {
                map.remove(key);
            }
        }
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.entries.lock() {
            self.epoch.fetch_add(1, Ordering::AcqRel);
            if let Some(map) = guard.as_mut() {
                map.clear();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serves_values_within_ttl_and_drops_them_after_invalidation() {
        let cache: TtlCache<String> = TtlCache::new(Duration::from_secs(60));
        assert!(cache.get("k").is_none());
        cache.put("k", "v1".into(), cache.epoch());
        assert_eq!(cache.get("k").as_deref(), Some("v1"));
        cache.put("k", "v2".into(), cache.epoch());
        assert_eq!(cache.get("k").as_deref(), Some("v2"));
        cache.invalidate("k");
        assert!(cache.get("k").is_none());
    }

    #[test]
    fn expired_entries_are_not_served() {
        let cache: TtlCache<u8> = TtlCache::new(Duration::from_millis(10));
        cache.put("k", 7, cache.epoch());
        std::thread::sleep(Duration::from_millis(30));
        assert!(cache.get("k").is_none());
    }

    #[test]
    fn a_probe_started_before_a_clear_cannot_store_its_result() {
        let cache: TtlCache<u8> = TtlCache::new(Duration::from_secs(60));
        let before_install = cache.epoch();
        cache.clear();
        cache.put("k", 1, before_install);
        assert!(cache.get("k").is_none());
        cache.put("k", 2, cache.epoch());
        assert_eq!(cache.get("k"), Some(2));
    }
}
