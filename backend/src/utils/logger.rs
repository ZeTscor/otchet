use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info, warn};

#[derive(Debug)]
pub struct StructuredLogger;

impl StructuredLogger {
    pub fn new() -> Self {
        Self
    }

    pub fn log_request(&self, method: &str, path: &str, user_id: Option<i32>, status: u16) {
        let log_entry = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "event_type": "http_request",
            "method": method,
            "path": path,
            "user_id": user_id,
            "status_code": status,
            "service": "job-tracker-backend"
        });

        info!("{}", log_entry);
    }

    pub fn log_database_query(&self, query: &str, duration_ms: u128, result_count: Option<usize>) {
        let log_entry = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "event_type": "database_query",
            "query_hash": format!("{:x}", md5::compute(query)),
            "query_preview": if query.len() > 100 {
                format!("{}...", &query[..100])
            } else {
                query.to_string()
            },
            "duration_ms": duration_ms,
            "result_count": result_count,
            "service": "job-tracker-backend"
        });

        if duration_ms > 1000 {
            warn!("Slow query detected: {}", log_entry);
        } else {
            info!("{}", log_entry);
        }
    }

    pub fn log_error(&self, error: &str, context: HashMap<String, serde_json::Value>) {
        let mut log_entry = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "event_type": "error",
            "error_message": error,
            "service": "job-tracker-backend"
        });

        for (key, value) in context {
            log_entry[key] = value;
        }

        error!("{}", log_entry);
    }

    pub fn log_performance_metric(
        &self,
        metric_name: &str,
        value: f64,
        tags: HashMap<String, String>,
    ) {
        let log_entry = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "event_type": "performance_metric",
            "metric_name": metric_name,
            "value": value,
            "tags": tags,
            "service": "job-tracker-backend"
        });

        info!("{}", log_entry);
    }

    pub fn log_business_event(
        &self,
        event_name: &str,
        user_id: Option<i32>,
        metadata: HashMap<String, serde_json::Value>,
    ) {
        let mut log_entry = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "event_type": "business_event",
            "event_name": event_name,
            "user_id": user_id,
            "service": "job-tracker-backend"
        });

        for (key, value) in metadata {
            log_entry[key] = value;
        }

        info!("{}", log_entry);
    }
}

pub static LOGGER: StructuredLogger = StructuredLogger;
