use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub timestamp: u64,
    pub data: Option<T>,
}

#[derive(Debug)]
pub struct DiskCache<T: Serialize + for<'de> Deserialize<'de> + Clone> {
    cache_file: PathBuf,
    entries: HashMap<String, CacheEntry<T>>,
    positive_ttl_secs: u64,
    negative_ttl_secs: u64,
}

impl<T: Serialize + for<'de> Deserialize<'de> + Clone> DiskCache<T> {
    pub fn new(filename: &str) -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("watznue");
        
        let _ = fs::create_dir_all(&cache_dir);
        let cache_file = cache_dir.join(filename);

        let mut cache = Self {
            cache_file,
            entries: HashMap::new(),
            positive_ttl_secs: 12 * 3600, // 12 hours
            negative_ttl_secs: 6 * 3600,  // 6 hours
        };
        cache.load();
        cache
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn load(&mut self) {
        if self.cache_file.exists() {
            if let Ok(file) = File::open(&self.cache_file) {
                let reader = BufReader::new(file);
                if let Ok(loaded) = serde_json::from_reader(reader) {
                    self.entries = loaded;
                }
            }
        }
    }

    pub fn save(&self) {
        if let Ok(serialized) = serde_json::to_string_pretty(&self.entries) {
            let _ = fs::write(&self.cache_file, serialized);
        }
    }

    pub fn get(&self, key: &str) -> Option<Option<T>> {
        if let Some(entry) = self.entries.get(key) {
            let now = Self::now_secs();
            let age = now.saturating_sub(entry.timestamp);

            let is_valid = match &entry.data {
                Some(_) => age < self.positive_ttl_secs,
                None => age < self.negative_ttl_secs,
            };

            if is_valid {
                return Some(entry.data.clone());
            }
        }
        None
    }

    pub fn insert(&mut self, key: impl Into<String>, value: Option<T>) {
        let entry = CacheEntry {
            timestamp: Self::now_secs(),
            data: value,
        };
        self.entries.insert(key.into(), entry);
        self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_and_get_positive() {
        let mut cache: DiskCache<String> = DiskCache::new("test_cache_pos.json");
        cache.insert("pkg_a", Some("advisory_data_123".to_string()));

        let retrieved = cache.get("pkg_a");
        assert_eq!(retrieved, Some(Some("advisory_data_123".to_string())));
    }

    #[test]
    fn test_cache_insert_and_get_negative() {
        let mut cache: DiskCache<String> = DiskCache::new("test_cache_neg.json");
        cache.insert("non_existent_pkg", None);

        let retrieved = cache.get("non_existent_pkg");
        assert_eq!(retrieved, Some(None));
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache: DiskCache<String> = DiskCache::new("test_cache_exp.json");
        // Insert with expired timestamp (beyond 12 hours)
        let old_timestamp = DiskCache::<String>::now_secs().saturating_sub(15 * 3600);
        cache.entries.insert(
            "expired_pkg".to_string(),
            CacheEntry {
                timestamp: old_timestamp,
                data: Some("old_data".to_string()),
            },
        );

        let retrieved = cache.get("expired_pkg");
        assert_eq!(retrieved, None);
    }
}
