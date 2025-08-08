use std::time::Instant;
use std::collections::HashMap;
use sqlx::{PgPool, Row};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use crate::utils::logger::LOGGER;

/// Production-ready caching service with multiple storage backends
#[derive(Debug)]
pub struct CacheService {
    pool: PgPool,
    in_memory_cache: std::sync::RwLock<HashMap<String, CacheEntry>>,
    max_memory_entries: usize,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: serde_json::Value,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    hit_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_keys: usize,
    pub expired_keys: usize,
    pub memory_usage_mb: f64,
    pub hit_ratio: f64,
    pub average_retrieval_time_ms: f64,
}

#[derive(Debug)]
pub enum CacheError {
    SerializationError(String),
    DatabaseError(String),
    NotFound,
}

impl CacheService {
    pub fn new(pool: PgPool, max_memory_entries: usize) -> Self {
        Self {
            pool,
            in_memory_cache: std::sync::RwLock::new(HashMap::new()),
            max_memory_entries,
        }
    }

    /// Get value from cache with fallback strategy
    pub async fn get<T>(&self, key: &str) -> Result<T, CacheError> 
    where
        T: for<'de> Deserialize<'de>,
    {
        let start_time = Instant::now();
        
        // Try memory cache first (fastest)
        if let Ok(value) = self.get_from_memory(key) {
            self.log_cache_hit("memory", key, start_time.elapsed().as_millis() as f64);
            return serde_json::from_value(value)
                .map_err(|e| CacheError::SerializationError(e.to_string()));
        }

        // Try database cache (persistent)
        match self.get_from_database(key).await {
            Ok(value) => {
                // Store in memory for next time
                self.store_in_memory(key, &value, Duration::minutes(30));
                
                self.log_cache_hit("database", key, start_time.elapsed().as_millis() as f64);
                serde_json::from_value(value)
                    .map_err(|e| CacheError::SerializationError(e.to_string()))
            }
            Err(_) => {
                self.log_cache_miss(key, start_time.elapsed().as_millis() as f64);
                Err(CacheError::NotFound)
            }
        }
    }

    /// Store value in cache with TTL
    pub async fn set<T>(&self, key: &str, value: &T, ttl: Duration) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        // Store in both memory and database
        self.store_in_memory(key, &json_value, ttl);
        self.store_in_database(key, &json_value, ttl).await?;

        LOGGER.log_business_event(
            "cache_set",
            None,
            [
                ("cache_key".to_string(), serde_json::Value::String(key.to_string())),
                ("ttl_seconds".to_string(), serde_json::Value::Number(serde_json::Number::from(ttl.num_seconds())))
            ].iter().cloned().collect()
        );

        Ok(())
    }

    /// Get or compute value with caching
    pub async fn get_or_compute<T, F, Fut>(&self, key: &str, ttl: Duration, compute_fn: F) -> Result<T, CacheError>
    where
        T: Serialize + for<'de> Deserialize<'de>,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, CacheError>>,
    {
        // Try to get from cache first
        match self.get::<T>(key).await {
            Ok(value) => Ok(value),
            Err(CacheError::NotFound) => {
                // Compute new value
                let computed_value = compute_fn().await?;
                
                // Store in cache
                self.set(key, &computed_value, ttl).await?;
                
                LOGGER.log_business_event(
                    "cache_computed",
                    None,
                    [(
                        "cache_key".to_string(),
                        serde_json::Value::String(key.to_string())
                    )].iter().cloned().collect()
                );
                
                Ok(computed_value)
            }
            Err(e) => Err(e),
        }
    }

    /// Invalidate cache key
    pub async fn invalidate(&self, key: &str) -> Result<(), CacheError> {
        // Remove from memory
        if let Ok(mut cache) = self.in_memory_cache.write() {
            cache.remove(key);
        }

        // Remove from database
        sqlx::query("DELETE FROM cache_store WHERE key = $1")
            .bind(key)
            .execute(&self.pool)
            .await
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        LOGGER.log_business_event(
            "cache_invalidated",
            None,
            [(
                "cache_key".to_string(),
                serde_json::Value::String(key.to_string())
            )].iter().cloned().collect()
        );

        Ok(())
    }

    /// Invalidate cache keys matching pattern
    pub async fn invalidate_pattern(&self, pattern: &str) -> Result<usize, CacheError> {
        let mut invalidated = 0;

        // Remove from memory cache
        if let Ok(mut cache) = self.in_memory_cache.write() {
            let keys_to_remove: Vec<String> = cache
                .keys()
                .filter(|k| k.contains(pattern))
                .cloned()
                .collect();
            
            for key in keys_to_remove {
                cache.remove(&key);
                invalidated += 1;
            }
        }

        // Remove from database cache
        let db_result = sqlx::query("DELETE FROM cache_store WHERE key LIKE $1")
            .bind(format!("%{}%", pattern))
            .execute(&self.pool)
            .await
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        invalidated += db_result.rows_affected() as usize;

        LOGGER.log_business_event(
            "cache_pattern_invalidated",
            None,
            [
                ("pattern".to_string(), serde_json::Value::String(pattern.to_string())),
                ("invalidated_count".to_string(), serde_json::Value::Number(serde_json::Number::from(invalidated)))
            ].iter().cloned().collect()
        );

        Ok(invalidated)
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> Result<CacheStats, CacheError> {
        // Memory cache stats
        let (memory_keys, total_hits, avg_retrieval) = {
            if let Ok(cache) = self.in_memory_cache.read() {
                let keys = cache.len();
                let total_hits: u64 = cache.values().map(|entry| entry.hit_count).sum();
                (keys, total_hits, 0.5) // Memory access is very fast
            } else {
                (0, 0, 0.0)
            }
        };

        // Database cache stats
        let db_stats = sqlx::query(
            "SELECT 
                COUNT(*) as total_keys,
                COUNT(CASE WHEN expires_at < NOW() THEN 1 END) as expired_keys
             FROM cache_store"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        let db_keys: i64 = db_stats.get(0);
        let expired_keys: i64 = db_stats.get(1);

        let total_keys = memory_keys + db_keys as usize;
        let hit_ratio = if total_keys > 0 {
            total_hits as f64 / total_keys as f64
        } else {
            0.0
        };

        Ok(CacheStats {
            total_keys,
            expired_keys: expired_keys as usize,
            memory_usage_mb: (memory_keys * 1024) as f64 / 1024.0 / 1024.0, // Rough estimate
            hit_ratio,
            average_retrieval_time_ms: avg_retrieval,
        })
    }

    /// Clean expired entries
    pub async fn cleanup_expired(&self) -> Result<usize, CacheError> {
        let mut cleaned = 0;

        // Clean memory cache
        if let Ok(mut cache) = self.in_memory_cache.write() {
            let now = Utc::now();
            let expired_keys: Vec<String> = cache
                .iter()
                .filter(|(_, entry)| entry.expires_at < now)
                .map(|(key, _)| key.clone())
                .collect();

            for key in expired_keys {
                cache.remove(&key);
                cleaned += 1;
            }
        }

        // Clean database cache
        let db_result = sqlx::query("DELETE FROM cache_store WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        cleaned += db_result.rows_affected() as usize;

        if cleaned > 0 {
            LOGGER.log_business_event(
                "cache_cleanup_completed",
                None,
                [(
                    "cleaned_entries".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(cleaned))
                )].iter().cloned().collect()
            );
        }

        Ok(cleaned)
    }

    /// Warm cache with commonly accessed data
    pub async fn warm_cache(&self) -> Result<(), CacheError> {
        let start_time = Instant::now();

        // Pre-cache commonly accessed analytics
        let _ = self.get_or_compute("analytics_summary", Duration::minutes(30), || async {
            // Simulate analytics computation
            Ok(serde_json::json!({
                "total_students": 0,
                "total_applications": 0,
                "cached": true
            }))
        }).await;

        // Pre-cache user activity patterns
        let _ = self.get_or_compute("activity_patterns", Duration::minutes(15), || async {
            Ok(serde_json::json!({
                "patterns": [],
                "cached": true
            }))
        }).await;

        let duration = start_time.elapsed();
        LOGGER.log_performance_metric(
            "cache_warmup", 
            duration.as_millis() as f64,
            HashMap::new()
        );

        Ok(())
    }

    // Private helper methods

    fn get_from_memory(&self, key: &str) -> Result<serde_json::Value, CacheError> {
        if let Ok(mut cache) = self.in_memory_cache.write() {
            if let Some(entry) = cache.get_mut(key) {
                if entry.expires_at > Utc::now() {
                    entry.hit_count += 1;
                    return Ok(entry.value.clone());
                } else {
                    // Remove expired entry
                    cache.remove(key);
                }
            }
        }
        Err(CacheError::NotFound)
    }

    fn store_in_memory(&self, key: &str, value: &serde_json::Value, ttl: Duration) {
        if let Ok(mut cache) = self.in_memory_cache.write() {
            // Evict old entries if at capacity
            if cache.len() >= self.max_memory_entries {
                // Simple LRU: remove oldest entry
                if let Some(oldest_key) = cache
                    .iter()
                    .min_by_key(|(_, entry)| entry.created_at)
                    .map(|(key, _)| key.clone())
                {
                    cache.remove(&oldest_key);
                }
            }

            let entry = CacheEntry {
                value: value.clone(),
                expires_at: Utc::now() + ttl,
                created_at: Utc::now(),
                hit_count: 0,
            };

            cache.insert(key.to_string(), entry);
        }
    }

    async fn get_from_database(&self, key: &str) -> Result<serde_json::Value, CacheError> {
        let row = sqlx::query(
            "SELECT value FROM cache_store WHERE key = $1 AND expires_at > NOW()"
        )
        .bind(key)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| CacheError::NotFound)?;

        Ok(row.get(0))
    }

    async fn store_in_database(&self, key: &str, value: &serde_json::Value, ttl: Duration) -> Result<(), CacheError> {
        let expires_at = Utc::now() + ttl;

        sqlx::query(
            "INSERT INTO cache_store (key, value, expires_at) 
             VALUES ($1, $2, $3) 
             ON CONFLICT (key) DO UPDATE SET value = $2, expires_at = $3"
        )
        .bind(key)
        .bind(value)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    fn log_cache_hit(&self, cache_type: &str, key: &str, duration_ms: f64) {
        LOGGER.log_performance_metric(
            "cache_hit", 
            duration_ms,
            [
                ("cache_type".to_string(), cache_type.to_string()),
                ("cache_key".to_string(), key.to_string())
            ].iter().cloned().collect()
        );
    }

    fn log_cache_miss(&self, key: &str, duration_ms: f64) {
        LOGGER.log_performance_metric(
            "cache_miss", 
            duration_ms,
            [("cache_key".to_string(), key.to_string())].iter().cloned().collect()
        );
    }
}

/// Cache invalidation strategies
pub enum InvalidationStrategy {
    TimeToLive(Duration),
    WriteThrough,
    WriteBehind(Duration),
    Manual,
}

/// Cache wrapper for specific data types
pub struct TypedCache<T> {
    cache: std::sync::Arc<CacheService>,
    key_prefix: String,
    default_ttl: Duration,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> TypedCache<T> 
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(cache: std::sync::Arc<CacheService>, key_prefix: String, default_ttl: Duration) -> Self {
        Self {
            cache,
            key_prefix,
            default_ttl,
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn get(&self, id: &str) -> Result<T, CacheError> {
        let key = format!("{}:{}", self.key_prefix, id);
        self.cache.get(&key).await
    }

    pub async fn set(&self, id: &str, value: &T) -> Result<(), CacheError> {
        let key = format!("{}:{}", self.key_prefix, id);
        self.cache.set(&key, value, self.default_ttl).await
    }

    pub async fn get_or_compute<F, Fut>(&self, id: &str, compute_fn: F) -> Result<T, CacheError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, CacheError>>,
    {
        let key = format!("{}:{}", self.key_prefix, id);
        self.cache.get_or_compute(&key, self.default_ttl, compute_fn).await
    }

    pub async fn invalidate(&self, id: &str) -> Result<(), CacheError> {
        let key = format!("{}:{}", self.key_prefix, id);
        self.cache.invalidate(&key).await
    }

    pub async fn invalidate_all(&self) -> Result<usize, CacheError> {
        self.cache.invalidate_pattern(&self.key_prefix).await
    }
}