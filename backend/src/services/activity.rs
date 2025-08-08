use std::time::Instant;
use std::collections::HashMap;
use sqlx::{PgPool, Row};
use serde::Serialize;
use crate::utils::logger::LOGGER;

#[derive(Debug, Serialize)]
pub struct ActivityData {
    pub date: String,
    pub applications_count: i32,
    pub screenings_count: i32,
    pub interviews_count: i32,
    pub total_activity: i32,
}

#[derive(Debug)]
pub enum ActivityError {
    DatabaseError(String),
    PermissionDenied,
}

#[derive(Debug)]
pub struct ActivityService {
    pool: PgPool,
}

impl ActivityService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get simplified user activity for the last year
    pub async fn get_user_activity(&self, user_id: i32) -> Result<Vec<ActivityData>, ActivityError> {
        let start_time = Instant::now();
        
        LOGGER.log_business_event(
            "user_activity_request_started", 
            Some(user_id), 
            HashMap::new()
        );

        let query = r#"
            SELECT 
                CURRENT_DATE - s.i AS date,
                COALESCE(a.applications_count, 0) as applications_count,
                COALESCE(sc.screenings_count, 0) as screenings_count,
                COALESCE(i.interviews_count, 0) as interviews_count
            FROM generate_series(0, 364) AS s(i)
            LEFT JOIN (
                SELECT DATE(created_at) as date, COUNT(*)::int as applications_count
                FROM applications 
                WHERE user_id = $1 
                GROUP BY DATE(created_at)
            ) a ON CURRENT_DATE - s.i = a.date
            LEFT JOIN (
                SELECT DATE(s.created_at) as date, COUNT(*)::int as screenings_count
                FROM screenings s
                JOIN applications ap ON s.application_id = ap.id
                WHERE ap.user_id = $1
                GROUP BY DATE(s.created_at)
            ) sc ON CURRENT_DATE - s.i = sc.date
            LEFT JOIN (
                SELECT DATE(i.created_at) as date, COUNT(*)::int as interviews_count
                FROM interviews i
                JOIN applications ap ON i.application_id = ap.id
                WHERE ap.user_id = $1
                GROUP BY DATE(i.created_at)
            ) i ON CURRENT_DATE - s.i = i.date
            ORDER BY date
        "#;

        let rows = sqlx::query(query)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ActivityError::DatabaseError(e.to_string()))?;

        let activity_data: Vec<ActivityData> = rows.iter().map(|row| {
            let applications_count: i32 = row.get(1);
            let screenings_count: i32 = row.get(2);
            let interviews_count: i32 = row.get(3);
            let total_activity = applications_count + screenings_count + interviews_count;

            ActivityData {
                date: row.get::<chrono::NaiveDate, _>(0).to_string(),
                applications_count,
                screenings_count,
                interviews_count,
                total_activity,
            }
        }).collect();

        let duration = start_time.elapsed();
        LOGGER.log_database_query(
            query, 
            duration.as_millis(), 
            Some(activity_data.len())
        );

        LOGGER.log_business_event(
            "user_activity_request_completed", 
            Some(user_id),
            [(
                "activity_days".to_string(), 
                serde_json::Value::Number(serde_json::Number::from(activity_data.len()))
            )].iter().cloned().collect()
        );

        Ok(activity_data)
    }

    /// Get admin activity overview (all users)
    pub async fn get_admin_activity(&self) -> Result<Vec<ActivityData>, ActivityError> {
        let start_time = Instant::now();
        
        LOGGER.log_business_event(
            "admin_activity_request_started",
            None,
            HashMap::new()
        );

        let query = r#"
            SELECT 
                CURRENT_DATE - s.i AS date,
                COALESCE(a.applications_count, 0) as applications_count,
                COALESCE(sc.screenings_count, 0) as screenings_count,
                COALESCE(i.interviews_count, 0) as interviews_count
            FROM generate_series(0, 364) AS s(i)
            LEFT JOIN (
                SELECT DATE(created_at) as date, COUNT(*)::int as applications_count
                FROM applications 
                GROUP BY DATE(created_at)
            ) a ON CURRENT_DATE - s.i = a.date
            LEFT JOIN (
                SELECT DATE(s.created_at) as date, COUNT(*)::int as screenings_count
                FROM screenings s
                GROUP BY DATE(s.created_at)
            ) sc ON CURRENT_DATE - s.i = sc.date
            LEFT JOIN (
                SELECT DATE(i.created_at) as date, COUNT(*)::int as interviews_count
                FROM interviews i
                GROUP BY DATE(i.created_at)
            ) i ON CURRENT_DATE - s.i = i.date
            ORDER BY date
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ActivityError::DatabaseError(e.to_string()))?;

        let activity_data: Vec<ActivityData> = rows.iter().map(|row| {
            let applications_count: i32 = row.get(1);
            let screenings_count: i32 = row.get(2);
            let interviews_count: i32 = row.get(3);
            let total_activity = applications_count + screenings_count + interviews_count;

            ActivityData {
                date: row.get::<chrono::NaiveDate, _>(0).to_string(),
                applications_count,
                screenings_count,
                interviews_count,
                total_activity,
            }
        }).collect();

        let duration = start_time.elapsed();
        LOGGER.log_database_query(
            query, 
            duration.as_millis(), 
            Some(activity_data.len())
        );

        LOGGER.log_business_event(
            "admin_activity_request_completed",
            None,
            [(
                "activity_days".to_string(), 
                serde_json::Value::Number(serde_json::Number::from(activity_data.len()))
            )].iter().cloned().collect()
        );

        Ok(activity_data)
    }
}