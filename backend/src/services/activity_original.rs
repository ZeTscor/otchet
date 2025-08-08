use std::time::Instant;
use std::collections::HashMap;
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::utils::logger::LOGGER;
use chrono::{DateTime, Utc, Duration, NaiveDate};

#[derive(Debug, Serialize)]
pub struct ActivityData {
    pub date: String,
    pub applications_count: i32,
    pub screenings_count: i32,
    pub interviews_count: i32,
    pub total_activity: i32,
}

#[derive(Debug)]
pub struct ActivityService {
    pool: PgPool,
}

impl ActivityService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get optimized user activity for the last year
    pub async fn get_user_activity(&self, user_id: i32) -> Result<Vec<ActivityData>, ActivityError> {
        let start_time = Instant::now();
        
        LOGGER.log_business_event(
            "user_activity_request_started", 
            Some(user_id), 
            HashMap::new()
        );

        // Use a more efficient approach: get actual activity data and fill gaps
        let result = sqlx::query!(
            r#"
            WITH date_range AS (
                SELECT generate_series(
                    CURRENT_DATE - INTERVAL '365 days',
                    CURRENT_DATE,
                    INTERVAL '1 day'
                )::date AS date
            ),
            daily_applications AS (
                SELECT DATE(created_at) as date, COUNT(*)::int as applications_count
                FROM applications 
                WHERE user_id = $1 AND created_at >= CURRENT_DATE - INTERVAL '365 days'
                GROUP BY DATE(created_at)
            ),
            daily_screenings AS (
                SELECT DATE(s.created_at) as date, COUNT(*)::int as screenings_count
                FROM screenings s
                JOIN applications a ON s.application_id = a.id
                WHERE a.user_id = $1 AND s.created_at >= CURRENT_DATE - INTERVAL '365 days'
                GROUP BY DATE(s.created_at)
            ),
            daily_interviews AS (
                SELECT DATE(i.created_at) as date, COUNT(*)::int as interviews_count
                FROM interviews i
                JOIN applications a ON i.application_id = a.id
                WHERE a.user_id = $1 AND i.created_at >= CURRENT_DATE - INTERVAL '365 days'
                GROUP BY DATE(i.created_at)
            )
            SELECT 
                dr.date::text,
                COALESCE(da.applications_count, 0) as applications_count,
                COALESCE(ds.screenings_count, 0) as screenings_count,
                COALESCE(di.interviews_count, 0) as interviews_count
            FROM date_range dr
            LEFT JOIN daily_applications da ON dr.date = da.date
            LEFT JOIN daily_screenings ds ON dr.date = ds.date
            LEFT JOIN daily_interviews di ON dr.date = di.date
            ORDER BY dr.date
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await;

        let duration = start_time.elapsed();
        LOGGER.log_database_query(
            "get_user_activity_optimized",
            duration.as_millis(),
            result.as_ref().map(|r| r.len()).ok()
        );

        match result {
            Ok(rows) => {
                let activity_data: Vec<ActivityData> = rows
                    .into_iter()
                    .map(|row| {
                        let applications_count = row.applications_count.unwrap_or(0);
                        let screenings_count = row.screenings_count.unwrap_or(0);
                        let interviews_count = row.interviews_count.unwrap_or(0);
                        let total_activity = applications_count + screenings_count + interviews_count;
                        
                        ActivityData {
                            date: row.date.unwrap_or_default(),
                            applications_count,
                            screenings_count,
                            interviews_count,
                            total_activity,
                        }
                    })
                    .collect();

                LOGGER.log_performance_metric(
                    "user_activity_request_duration",
                    duration.as_millis() as f64,
                    [
                        ("operation".to_string(), "get_user_activity".to_string()),
                        ("user_id".to_string(), user_id.to_string()),
                        ("result_count".to_string(), activity_data.len().to_string())
                    ].iter().cloned().collect()
                );

                Ok(activity_data)
            }
            Err(e) => {
                let mut context = HashMap::new();
                context.insert("user_id".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(user_id)
                ));
                context.insert("duration_ms".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(duration.as_millis() as u64)
                ));
                LOGGER.log_error(&format!("User activity query failed: {}", e), context);
                Err(ActivityError::DatabaseError(e.to_string()))
            }
        }
    }

    /// Get optimized admin activity (all users) for the last year
    pub async fn get_admin_activity(&self) -> Result<Vec<ActivityData>, ActivityError> {
        let start_time = Instant::now();
        
        LOGGER.log_business_event(
            "admin_activity_request_started", 
            None, 
            HashMap::new()
        );

        // More efficient query for admin overview
        let result = sqlx::query!(
            r#"
            WITH date_range AS (
                SELECT generate_series(
                    CURRENT_DATE - INTERVAL '365 days',
                    CURRENT_DATE,
                    INTERVAL '1 day'
                )::date AS date
            ),
            daily_applications AS (
                SELECT DATE(created_at) as date, COUNT(*)::int as applications_count
                FROM applications 
                WHERE created_at >= CURRENT_DATE - INTERVAL '365 days'
                GROUP BY DATE(created_at)
            ),
            daily_screenings AS (
                SELECT DATE(created_at) as date, COUNT(*)::int as screenings_count
                FROM screenings
                WHERE created_at >= CURRENT_DATE - INTERVAL '365 days'
                GROUP BY DATE(created_at)
            ),
            daily_interviews AS (
                SELECT DATE(created_at) as date, COUNT(*)::int as interviews_count
                FROM interviews
                WHERE created_at >= CURRENT_DATE - INTERVAL '365 days'
                GROUP BY DATE(created_at)
            )
            SELECT 
                dr.date::text,
                COALESCE(da.applications_count, 0) as applications_count,
                COALESCE(ds.screenings_count, 0) as screenings_count,
                COALESCE(di.interviews_count, 0) as interviews_count
            FROM date_range dr
            LEFT JOIN daily_applications da ON dr.date = da.date
            LEFT JOIN daily_screenings ds ON dr.date = ds.date
            LEFT JOIN daily_interviews di ON dr.date = di.date
            ORDER BY dr.date
            "#
        )
        .fetch_all(&self.pool)
        .await;

        let duration = start_time.elapsed();
        LOGGER.log_database_query(
            "get_admin_activity_optimized",
            duration.as_millis(),
            result.as_ref().map(|r| r.len()).ok()
        );

        match result {
            Ok(rows) => {
                let activity_data: Vec<ActivityData> = rows
                    .into_iter()
                    .map(|row| {
                        let applications_count = row.applications_count.unwrap_or(0);
                        let screenings_count = row.screenings_count.unwrap_or(0);
                        let interviews_count = row.interviews_count.unwrap_or(0);
                        let total_activity = applications_count + screenings_count + interviews_count;
                        
                        ActivityData {
                            date: row.date.unwrap_or_default(),
                            applications_count,
                            screenings_count,
                            interviews_count,
                            total_activity,
                        }
                    })
                    .collect();

                LOGGER.log_performance_metric(
                    "admin_activity_request_duration",
                    duration.as_millis() as f64,
                    [
                        ("operation".to_string(), "get_admin_activity".to_string()),
                        ("result_count".to_string(), activity_data.len().to_string())
                    ].iter().cloned().collect()
                );

                Ok(activity_data)
            }
            Err(e) => {
                let mut context = HashMap::new();
                context.insert("duration_ms".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(duration.as_millis() as u64)
                ));
                LOGGER.log_error(&format!("Admin activity query failed: {}", e), context);
                Err(ActivityError::DatabaseError(e.to_string()))
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ActivityError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Permission denied")]
    PermissionDenied,
}

impl From<sqlx::Error> for ActivityError {
    fn from(err: sqlx::Error) -> Self {
        ActivityError::DatabaseError(err.to_string())
    }
}