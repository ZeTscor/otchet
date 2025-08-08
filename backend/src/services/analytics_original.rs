use std::time::Instant;
use std::collections::HashMap;
use sqlx::PgPool;
use serde::Serialize;
use crate::utils::logger::LOGGER;
use crate::handlers::admin::*;
use crate::models::application::ApplicationResponse;

#[derive(Debug)]
pub struct AnalyticsService {
    pool: PgPool,
}

impl AnalyticsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_comprehensive_analytics(&self) -> Result<AnalyticsResponse, AnalyticsError> {
        let start_time = Instant::now();
        
        LOGGER.log_business_event("analytics_request_started", None, HashMap::new());

        // Execute all queries in parallel for better performance
        let results = tokio::try_join!(
            self.get_basic_counts(),
            self.get_status_breakdown(),
            self.get_company_stats(),
            self.get_job_url_stats(),
            self.get_screening_stats(),
            self.get_interview_stats(),
            self.get_success_rates(),
            self.get_stale_applications()
        );

        match results {
            Ok((
                (total_students, total_applications),
                status_breakdown,
                company_stats,
                popular_job_urls,
                screening_stats,
                interview_stats,
                success_rate,
                stale_applications
            )) => {
                let response = AnalyticsResponse {
                    total_students,
                    total_applications,
                    status_breakdown,
                    company_stats,
                    popular_job_urls,
                    stale_applications,
                    screening_stats,
                    interview_stats,
                    daily_stats: Vec::new(), // Simplified for now
                    success_rate,
                    response_times: ResponseTimeStats {
                        avg_days_to_screening: 0.0,
                        avg_days_to_interview: 0.0,
                        fastest_screening_days: 0,
                        slowest_screening_days: 0,
                    },
                    top_performing_students: Vec::new(), // Simplified for now
                };

                let duration = start_time.elapsed();
                LOGGER.log_performance_metric(
                    "analytics_request_duration", 
                    duration.as_millis() as f64,
                    [("operation".to_string(), "get_analytics".to_string())].iter().cloned().collect()
                );

                Ok(response)
            }
            Err(e) => {
                let mut context = HashMap::new();
                context.insert("duration_ms".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(start_time.elapsed().as_millis() as u64)
                ));
                LOGGER.log_error(&format!("Analytics query failed: {}", e), context);
                Err(AnalyticsError::DatabaseError(e.to_string()))
            }
        }
    }

    async fn get_basic_counts(&self) -> Result<(i64, i64), sqlx::Error> {
        let query_start = Instant::now();
        
        let result = sqlx::query!(
            "SELECT 
                (SELECT COUNT(*) FROM users WHERE role = 'student') as student_count,
                (SELECT COUNT(*) FROM applications) as application_count"
        )
        .fetch_one(&self.pool)
        .await;

        LOGGER.log_database_query(
            "SELECT basic counts", 
            query_start.elapsed().as_millis(),
            Some(1)
        );

        match result {
            Ok(row) => Ok((row.student_count.unwrap_or(0), row.application_count.unwrap_or(0))),
            Err(e) => Err(e)
        }
    }

    async fn get_status_breakdown(&self) -> Result<HashMap<String, i64>, sqlx::Error> {
        let query_start = Instant::now();
        
        let rows = sqlx::query!(
            "SELECT status::text as status, COUNT(*)::bigint as count 
             FROM applications 
             GROUP BY status"
        )
        .fetch_all(&self.pool)
        .await;

        LOGGER.log_database_query(
            "SELECT status breakdown", 
            query_start.elapsed().as_millis(),
            rows.as_ref().map(|r| r.len()).ok()
        );

        match rows {
            Ok(rows) => {
                let mut breakdown = HashMap::new();
                for row in rows {
                    if let (Some(status), Some(count)) = (row.status, row.count) {
                        breakdown.insert(status, count);
                    }
                }
                Ok(breakdown)
            }
            Err(e) => Err(e)
        }
    }

    async fn get_company_stats(&self) -> Result<Vec<CompanyStats>, sqlx::Error> {
        let query_start = Instant::now();
        
        let rows = sqlx::query!(
            "SELECT 
                company, 
                COUNT(*)::bigint as application_count, 
                COUNT(DISTINCT user_id)::bigint as unique_students
             FROM applications 
             GROUP BY company 
             ORDER BY application_count DESC 
             LIMIT 10"
        )
        .fetch_all(&self.pool)
        .await;

        LOGGER.log_database_query(
            "SELECT company stats", 
            query_start.elapsed().as_millis(),
            rows.as_ref().map(|r| r.len()).ok()
        );

        match rows {
            Ok(rows) => {
                let stats = rows.into_iter().map(|row| CompanyStats {
                    company: row.company,
                    application_count: row.application_count.unwrap_or(0),
                    unique_students: row.unique_students.unwrap_or(0),
                }).collect();
                Ok(stats)
            }
            Err(e) => Err(e)
        }
    }

    async fn get_job_url_stats(&self) -> Result<Vec<JobUrlStats>, sqlx::Error> {
        let query_start = Instant::now();
        
        let rows = sqlx::query!(
            "SELECT 
                job_url, 
                COUNT(*)::bigint as application_count, 
                COUNT(DISTINCT user_id)::bigint as unique_students
             FROM applications 
             WHERE job_url IS NOT NULL
             GROUP BY job_url 
             ORDER BY application_count DESC 
             LIMIT 10"
        )
        .fetch_all(&self.pool)
        .await;

        LOGGER.log_database_query(
            "SELECT job URL stats", 
            query_start.elapsed().as_millis(),
            rows.as_ref().map(|r| r.len()).ok()
        );

        match rows {
            Ok(rows) => {
                let stats = rows.into_iter().filter_map(|row| {
                    row.job_url.map(|url| JobUrlStats {
                        job_url: url,
                        application_count: row.application_count.unwrap_or(0),
                        unique_students: row.unique_students.unwrap_or(0),
                    })
                }).collect();
                Ok(stats)
            }
            Err(e) => Err(e)
        }
    }

    async fn get_screening_stats(&self) -> Result<ScreeningStats, sqlx::Error> {
        let query_start = Instant::now();
        
        let result = sqlx::query!(
            "SELECT 
                COUNT(*)::bigint as total,
                COUNT(CASE WHEN result = 'passed' THEN 1 END)::bigint as passed,
                COUNT(CASE WHEN result = 'failed' THEN 1 END)::bigint as failed
             FROM screenings"
        )
        .fetch_one(&self.pool)
        .await;

        LOGGER.log_database_query(
            "SELECT screening stats", 
            query_start.elapsed().as_millis(),
            Some(1)
        );

        match result {
            Ok(row) => {
                let total = row.total.unwrap_or(0);
                let passed = row.passed.unwrap_or(0);
                let failed = row.failed.unwrap_or(0);
                
                Ok(ScreeningStats {
                    total_screenings: total,
                    passed,
                    failed,
                    pending: total - passed - failed,
                })
            }
            Err(e) => Err(e)
        }
    }

    async fn get_interview_stats(&self) -> Result<InterviewStats, sqlx::Error> {
        let query_start = Instant::now();
        
        let result = sqlx::query!(
            "SELECT 
                COUNT(*)::bigint as total,
                COUNT(CASE WHEN result = 'passed' THEN 1 END)::bigint as passed,
                COUNT(CASE WHEN result = 'failed' THEN 1 END)::bigint as failed
             FROM interviews"
        )
        .fetch_one(&self.pool)
        .await;

        LOGGER.log_database_query(
            "SELECT interview stats", 
            query_start.elapsed().as_millis(),
            Some(1)
        );

        match result {
            Ok(row) => {
                let total = row.total.unwrap_or(0);
                let passed = row.passed.unwrap_or(0);
                let failed = row.failed.unwrap_or(0);
                
                Ok(InterviewStats {
                    total_interviews: total,
                    passed,
                    failed,
                    pending: total - passed - failed,
                })
            }
            Err(e) => Err(e)
        }
    }

    async fn get_success_rates(&self) -> Result<SuccessRateStats, sqlx::Error> {
        let query_start = Instant::now();
        
        let result = sqlx::query!(
            "SELECT 
                (SELECT COUNT(*)::bigint FROM applications) as total_apps,
                (SELECT COUNT(*)::bigint FROM interviews WHERE result = 'passed') as interview_passed,
                (SELECT COUNT(*)::bigint FROM screenings WHERE result = 'passed') as screening_passed,
                (SELECT COUNT(*)::bigint FROM interviews) as total_interviews,
                (SELECT COUNT(*)::bigint FROM applications WHERE job_url IS NOT NULL) as apps_with_urls,
                (SELECT COUNT(*)::bigint FROM applications WHERE job_url IS NULL) as apps_without_urls"
        )
        .fetch_one(&self.pool)
        .await;

        LOGGER.log_database_query(
            "SELECT success rates", 
            query_start.elapsed().as_millis(),
            Some(1)
        );

        match result {
            Ok(row) => {
                let total_apps = row.total_apps.unwrap_or(0);
                let interview_passed = row.interview_passed.unwrap_or(0);
                let screening_passed = row.screening_passed.unwrap_or(0);
                let total_interviews = row.total_interviews.unwrap_or(0);
                
                Ok(SuccessRateStats {
                    overall_success_rate: if total_apps > 0 {
                        (interview_passed as f64 / total_apps as f64) * 100.0
                    } else { 0.0 },
                    screening_to_interview_rate: if screening_passed > 0 {
                        (total_interviews as f64 / screening_passed as f64) * 100.0
                    } else { 0.0 },
                    interview_success_rate: if total_interviews > 0 {
                        (interview_passed as f64 / total_interviews as f64) * 100.0
                    } else { 0.0 },
                    applications_with_urls: row.apps_with_urls.unwrap_or(0),
                    applications_without_urls: row.apps_without_urls.unwrap_or(0),
                })
            }
            Err(e) => Err(e)
        }
    }

    async fn get_stale_applications(&self) -> Result<Vec<ApplicationResponse>, sqlx::Error> {
        let query_start = Instant::now();
        
        let rows = sqlx::query_as!(
            crate::models::application::Application,
            "SELECT * FROM applications 
             WHERE updated_at < NOW() - INTERVAL '7 days' 
             AND status IN ('waiting', 'next_stage') 
             ORDER BY updated_at ASC 
             LIMIT 5"
        )
        .fetch_all(&self.pool)
        .await;

        LOGGER.log_database_query(
            "SELECT stale applications", 
            query_start.elapsed().as_millis(),
            rows.as_ref().map(|r| r.len()).ok()
        );

        match rows {
            Ok(apps) => Ok(apps.into_iter().map(ApplicationResponse::from).collect()),
            Err(e) => Err(e)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AnalyticsError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Permission denied")]
    PermissionDenied,
}

impl From<sqlx::Error> for AnalyticsError {
    fn from(err: sqlx::Error) -> Self {
        AnalyticsError::DatabaseError(err.to_string())
    }
}