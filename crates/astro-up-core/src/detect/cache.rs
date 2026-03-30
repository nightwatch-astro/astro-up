use std::collections::HashMap;
use std::sync::RwLock;

use chrono::{DateTime, Utc};

use crate::detect::DetectionResult;

struct CacheEntry {
    result: DetectionResult,
    #[allow(dead_code)] // retained for future TTL support
    scanned_at: DateTime<Utc>,
}

/// In-memory detection cache with event-driven invalidation.
///
/// Thread-safe via RwLock. No TTL — entries live until explicitly invalidated.
pub struct DetectionCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
}

impl DetectionCache {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Get a cached detection result for a package.
    pub fn get(&self, package_id: &str) -> Option<DetectionResult> {
        self.entries
            .read()
            .ok()?
            .get(package_id)
            .map(|e| e.result.clone())
    }

    /// Insert or update a cached detection result.
    pub fn insert(&self, package_id: String, result: DetectionResult) {
        if let Ok(mut entries) = self.entries.write() {
            entries.insert(
                package_id,
                CacheEntry {
                    result,
                    scanned_at: Utc::now(),
                },
            );
        }
    }

    /// Invalidate cache for a specific package (e.g., after install/update).
    pub fn invalidate(&self, package_id: &str) {
        if let Ok(mut entries) = self.entries.write() {
            entries.remove(package_id);
        }
    }

    /// Invalidate entire cache (e.g., explicit scan command).
    pub fn invalidate_all(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.read().map(|e| e.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for DetectionCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DetectionMethod, Version};

    fn installed(v: &str) -> DetectionResult {
        DetectionResult::Installed {
            version: Version::parse(v),
            method: DetectionMethod::Registry,
        }
    }

    #[test]
    fn insert_and_get() {
        let cache = DetectionCache::new();
        cache.insert("nina".into(), installed("3.0.0"));
        assert!(cache.get("nina").is_some());
        assert!(cache.get("phd2").is_none());
    }

    #[test]
    fn invalidate_single() {
        let cache = DetectionCache::new();
        cache.insert("nina".into(), installed("3.0.0"));
        cache.insert("phd2".into(), installed("2.6.0"));
        cache.invalidate("nina");
        assert!(cache.get("nina").is_none());
        assert!(cache.get("phd2").is_some());
    }

    #[test]
    fn invalidate_all_clears() {
        let cache = DetectionCache::new();
        cache.insert("nina".into(), installed("3.0.0"));
        cache.insert("phd2".into(), installed("2.6.0"));
        cache.invalidate_all();
        assert!(cache.is_empty());
    }

    #[test]
    fn concurrent_reads() {
        use std::sync::Arc;

        let cache = Arc::new(DetectionCache::new());
        cache.insert("nina".into(), installed("3.0.0"));

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let c = Arc::clone(&cache);
                std::thread::spawn(move || c.get("nina").is_some())
            })
            .collect();

        for h in handles {
            assert!(h.join().unwrap());
        }
    }
}
