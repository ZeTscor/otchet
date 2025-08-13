use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    middleware::auth::AuthUser,
    services::cache::{CacheService, CacheStats},
    services::metrics::{MetricsError, MetricsService, TimeBasedMetrics},
    utils::logger::LOGGER,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    pub days: Option<i32>,
    pub cache_duration: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub metrics: TimeBasedMetrics,
    pub cached: bool,
    pub generation_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct CacheStatsResponse {
    pub cache_stats: CacheStats,
    pub cleanup_performed: bool,
    pub entries_cleaned: usize,
}

/// Get anonymized time-based metrics
pub async fn get_anonymous_metrics(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<MetricsResponse>, StatusCode> {
    // Only admins can access metrics
    if !auth_user.is_admin() {
        LOGGER.log_business_event(
            "unauthorized_metrics_access",
            Some(auth_user.user_id),
            [(
                "role".to_string(),
                serde_json::Value::String(auth_user.role_str().to_string()),
            )]
            .iter()
            .cloned()
            .collect(),
        );
        return Err(StatusCode::FORBIDDEN);
    }

    let days_back = query.days.unwrap_or(30);
    let cache_duration = query.cache_duration.unwrap_or(60); // 1 hour default

    LOGGER.log_request("GET", "/admin/metrics", Some(auth_user.user_id), 200);

    let start_time = std::time::Instant::now();
    let metrics_service = MetricsService::new(state.db.clone());

    match metrics_service
        .get_cached_metrics(days_back, cache_duration)
        .await
    {
        Ok(metrics) => {
            let generation_time = start_time.elapsed().as_millis() as u64;

            LOGGER.log_business_event(
                "anonymous_metrics_delivered",
                Some(auth_user.user_id),
                [
                    (
                        "days_back".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(days_back)),
                    ),
                    (
                        "generation_time_ms".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(generation_time)),
                    ),
                ]
                .iter()
                .cloned()
                .collect(),
            );

            Ok(Json(MetricsResponse {
                metrics,
                cached: generation_time < 100, // Assume cached if very fast
                generation_time_ms: generation_time,
            }))
        }
        Err(MetricsError::DatabaseError(msg)) => {
            let mut context = HashMap::new();
            context.insert(
                "user_id".to_string(),
                serde_json::Value::Number(serde_json::Number::from(auth_user.user_id)),
            );
            context.insert(
                "days_back".to_string(),
                serde_json::Value::Number(serde_json::Number::from(days_back)),
            );
            LOGGER.log_error(&msg, context);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(MetricsError::CalculationError(msg)) => {
            let mut context = HashMap::new();
            context.insert(
                "user_id".to_string(),
                serde_json::Value::Number(serde_json::Number::from(auth_user.user_id)),
            );
            LOGGER.log_error(&msg, context);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get cache statistics and perform cleanup
pub async fn get_cache_stats(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<CacheStatsResponse>, StatusCode> {
    // Only admins can access cache stats
    if !auth_user.is_admin() {
        LOGGER.log_business_event(
            "unauthorized_cache_access",
            Some(auth_user.user_id),
            [(
                "role".to_string(),
                serde_json::Value::String(auth_user.role_str().to_string()),
            )]
            .iter()
            .cloned()
            .collect(),
        );
        return Err(StatusCode::FORBIDDEN);
    }

    LOGGER.log_request("GET", "/admin/cache-stats", Some(auth_user.user_id), 200);

    // Create cache service with reasonable memory limits
    let cache_service = CacheService::new(state.db.clone(), 1000);

    let (stats_result, cleanup_result) =
        tokio::join!(cache_service.get_stats(), cache_service.cleanup_expired());

    match (stats_result, cleanup_result) {
        (Ok(cache_stats), Ok(entries_cleaned)) => {
            LOGGER.log_business_event(
                "cache_stats_delivered",
                Some(auth_user.user_id),
                [
                    (
                        "total_keys".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(cache_stats.total_keys)),
                    ),
                    (
                        "entries_cleaned".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(entries_cleaned)),
                    ),
                ]
                .iter()
                .cloned()
                .collect(),
            );

            Ok(Json(CacheStatsResponse {
                cache_stats,
                cleanup_performed: entries_cleaned > 0,
                entries_cleaned,
            }))
        }
        _ => {
            LOGGER.log_error("Failed to get cache statistics", HashMap::new());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Invalidate cache entries by pattern
#[derive(Debug, Deserialize)]
pub struct InvalidateRequest {
    pub pattern: String,
}

#[derive(Debug, Serialize)]
pub struct InvalidateResponse {
    pub invalidated_count: usize,
    pub success: bool,
}

pub async fn invalidate_cache(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<InvalidateRequest>,
) -> Result<Json<InvalidateResponse>, StatusCode> {
    // Only admins can invalidate cache
    if !auth_user.is_admin() {
        LOGGER.log_business_event(
            "unauthorized_cache_invalidation",
            Some(auth_user.user_id),
            [
                (
                    "role".to_string(),
                    serde_json::Value::String(auth_user.role_str().to_string()),
                ),
                (
                    "pattern".to_string(),
                    serde_json::Value::String(request.pattern.clone()),
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        );
        return Err(StatusCode::FORBIDDEN);
    }

    LOGGER.log_request(
        "POST",
        "/admin/cache-invalidate",
        Some(auth_user.user_id),
        200,
    );

    let cache_service = CacheService::new(state.db.clone(), 1000);

    match cache_service.invalidate_pattern(&request.pattern).await {
        Ok(invalidated_count) => {
            LOGGER.log_business_event(
                "cache_invalidated",
                Some(auth_user.user_id),
                [
                    (
                        "pattern".to_string(),
                        serde_json::Value::String(request.pattern),
                    ),
                    (
                        "invalidated_count".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(invalidated_count)),
                    ),
                ]
                .iter()
                .cloned()
                .collect(),
            );

            Ok(Json(InvalidateResponse {
                invalidated_count,
                success: true,
            }))
        }
        Err(_) => {
            LOGGER.log_error("Failed to invalidate cache", HashMap::new());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Warm cache with commonly accessed data
#[derive(Debug, Serialize)]
pub struct WarmCacheResponse {
    pub success: bool,
    pub warming_time_ms: u64,
}

pub async fn warm_cache(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<WarmCacheResponse>, StatusCode> {
    // Only admins can warm cache
    if !auth_user.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    LOGGER.log_request("POST", "/admin/cache-warm", Some(auth_user.user_id), 200);

    let start_time = std::time::Instant::now();
    let cache_service = CacheService::new(state.db.clone(), 1000);

    match cache_service.warm_cache().await {
        Ok(_) => {
            let warming_time = start_time.elapsed().as_millis() as u64;

            LOGGER.log_business_event(
                "cache_warmed",
                Some(auth_user.user_id),
                [(
                    "warming_time_ms".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(warming_time)),
                )]
                .iter()
                .cloned()
                .collect(),
            );

            Ok(Json(WarmCacheResponse {
                success: true,
                warming_time_ms: warming_time,
            }))
        }
        Err(_) => {
            LOGGER.log_error("Failed to warm cache", HashMap::new());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
