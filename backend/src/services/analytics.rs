use std::time::Instant;
use std::collections::HashMap;
use sqlx::{PgPool, Row};
use crate::utils::logger::LOGGER;
use crate::handlers::admin::*;
use crate::models::application::ApplicationResponse;

#[derive(Debug)]
pub struct AnalyticsService {
    pool: PgPool,
}

#[derive(Debug)]
pub enum AnalyticsError {
    DatabaseError(String),
    PermissionDenied,
}

impl AnalyticsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_comprehensive_analytics(&self) -> Result<AnalyticsResponse, AnalyticsError> {
        let start_time = Instant::now();

        LOGGER.log_business_event(
            "analytics_request_started",
            None,
            HashMap::new()
        );

        let results = tokio::try_join!(
            self.get_basic_counts(),
            self.get_status_breakdown(),
            self.get_company_stats(),
            self.get_popular_job_urls(),
            self.get_stale_applications(),
            self.get_screening_stats(),
            self.get_interview_stats(),
            self.get_success_rate_stats(),
            self.get_top_performing_students()
        );

        let duration = start_time.elapsed();
        LOGGER.log_performance_metric("analytics_total_duration", duration.as_millis() as f64, HashMap::new());

        match results {
            Ok((
                (total_students, total_applications),
                status_breakdown,
                company_stats,
                popular_job_urls,
                stale_applications,
                screening_stats,
                interview_stats,
                success_rate,
                top_performing_students
            )) => {
                let daily_stats = vec![]; // Simplified for now
                let response_times = ResponseTimeStats {
                    avg_days_to_screening: 0.0,
                    avg_days_to_interview: 0.0,
                    fastest_screening_days: 0,
                    slowest_screening_days: 0,
                };

                let analytics = AnalyticsResponse {
                    total_students,
                    total_applications,
                    status_breakdown,
                    company_stats,
                    popular_job_urls,
                    stale_applications,
                    screening_stats,
                    interview_stats,
                    daily_stats,
                    success_rate,
                    response_times,
                    top_performing_students,
                };

                LOGGER.log_business_event(
                    "analytics_request_completed",
                    None,
                    HashMap::new()
                );

                Ok(analytics)
            }
            Err(_) => Err(AnalyticsError::DatabaseError("Failed to fetch analytics".to_string()))
        }
    }

    async fn get_basic_counts(&self) -> Result<(i64, i64), sqlx::Error> {
        let row = sqlx::query(
            "SELECT 
                (SELECT COUNT(*)::bigint FROM users WHERE role = 'student') as students,
                (SELECT COUNT(*)::bigint FROM applications) as applications"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok((row.get(0), row.get(1)))
    }

    async fn get_status_breakdown(&self) -> Result<HashMap<String, i64>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT status::text, COUNT(*)::bigint as count 
             FROM applications 
             GROUP BY status"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut breakdown = HashMap::new();
        for row in rows {
            let status: String = row.get(0);
            let count: i64 = row.get(1);
            breakdown.insert(status, count);
        }

        Ok(breakdown)
    }

    async fn get_company_stats(&self) -> Result<Vec<CompanyStats>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT company, COUNT(*)::bigint as count, COUNT(DISTINCT user_id)::bigint as unique_students
             FROM applications 
             GROUP BY company 
             ORDER BY count DESC 
             LIMIT 10"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(CompanyStats {
                company: row.get(0),
                application_count: row.get(1),
                unique_students: row.get(2),
            });
        }

        Ok(stats)
    }

    async fn get_popular_job_urls(&self) -> Result<Vec<JobUrlStats>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT job_url, COUNT(*)::bigint as count, COUNT(DISTINCT user_id)::bigint as unique_students
             FROM applications 
             WHERE job_url IS NOT NULL
             GROUP BY job_url 
             ORDER BY count DESC 
             LIMIT 5"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(JobUrlStats {
                job_url: row.get(0),
                application_count: row.get(1),
                unique_students: row.get(2),
            });
        }

        Ok(stats)
    }

    async fn get_stale_applications(&self) -> Result<Vec<ApplicationResponse>, sqlx::Error> {
        let rows = sqlx::query_as::<_, crate::models::application::Application>(
            "SELECT * FROM applications 
             WHERE updated_at < NOW() - INTERVAL '7 days' 
               AND status NOT IN ('rejected', 'next_stage')
             ORDER BY updated_at ASC
             LIMIT 5"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(ApplicationResponse::from).collect())
    }

    async fn get_screening_stats(&self) -> Result<ScreeningStats, sqlx::Error> {
        let row = sqlx::query(
            "SELECT 
                COUNT(*)::bigint as total,
                COUNT(CASE WHEN result = 'passed' THEN 1 END)::bigint as passed,
                COUNT(CASE WHEN result = 'failed' THEN 1 END)::bigint as failed
             FROM screenings"
        )
        .fetch_one(&self.pool)
        .await?;

        let total: i64 = row.get(0);
        let passed: i64 = row.get(1);
        let failed: i64 = row.get(2);

        Ok(ScreeningStats {
            total_screenings: total,
            passed,
            failed,
            pending: total - passed - failed,
        })
    }

    async fn get_interview_stats(&self) -> Result<InterviewStats, sqlx::Error> {
        let row = sqlx::query(
            "SELECT 
                COUNT(*)::bigint as total,
                COUNT(CASE WHEN result = 'passed' THEN 1 END)::bigint as passed,
                COUNT(CASE WHEN result = 'failed' THEN 1 END)::bigint as failed
             FROM interviews"
        )
        .fetch_one(&self.pool)
        .await?;

        let total: i64 = row.get(0);
        let passed: i64 = row.get(1);
        let failed: i64 = row.get(2);

        Ok(InterviewStats {
            total_interviews: total,
            passed,
            failed,
            pending: total - passed - failed,
        })
    }

    async fn get_success_rate_stats(&self) -> Result<SuccessRateStats, sqlx::Error> {
        let row = sqlx::query(
            "SELECT 
                (SELECT COUNT(*)::bigint FROM applications) as total_apps,
                (SELECT COUNT(*)::bigint FROM interviews WHERE result = 'passed') as interview_passed,
                (SELECT COUNT(*)::bigint FROM screenings WHERE result = 'passed') as screening_passed,
                (SELECT COUNT(*)::bigint FROM applications WHERE job_url IS NOT NULL) as apps_with_urls,
                (SELECT COUNT(*)::bigint FROM applications WHERE job_url IS NULL) as apps_without_urls"
        )
        .fetch_one(&self.pool)
        .await?;

        let total_apps: i64 = row.get(0);
        let interview_passed: i64 = row.get(1);
        let screening_passed: i64 = row.get(2);
        let apps_with_urls: i64 = row.get(3);
        let apps_without_urls: i64 = row.get(4);

        let overall_success_rate = if total_apps > 0 {
            (interview_passed as f64 / total_apps as f64) * 100.0
        } else {
            0.0
        };

        let screening_to_interview_rate = if screening_passed > 0 {
            (interview_passed as f64 / screening_passed as f64) * 100.0
        } else {
            0.0
        };

        let interview_success_rate = if interview_passed > 0 {
            (interview_passed as f64 / (screening_passed as f64).max(1.0)) * 100.0
        } else {
            0.0
        };

        Ok(SuccessRateStats {
            overall_success_rate,
            screening_to_interview_rate,
            interview_success_rate,
            applications_with_urls: apps_with_urls,
            applications_without_urls: apps_without_urls,
        })
    }

    async fn get_top_performing_students(&self) -> Result<Vec<StudentPerformance>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT 
                u.email,
                u.first_name || ' ' || u.last_name as name,
                COUNT(a.id)::bigint as total_applications,
                COUNT(CASE WHEN s.result = 'passed' THEN 1 END)::bigint as screenings_passed,
                COUNT(CASE WHEN i.result = 'passed' THEN 1 END)::bigint as interviews_passed
             FROM users u
             LEFT JOIN applications a ON u.id = a.user_id
             LEFT JOIN screenings s ON a.id = s.application_id
             LEFT JOIN interviews i ON a.id = i.application_id
             WHERE u.role = 'student'
             GROUP BY u.id, u.email, u.first_name, u.last_name
             HAVING COUNT(a.id) > 0
             ORDER BY COUNT(CASE WHEN i.result = 'passed' THEN 1 END) DESC
             LIMIT 5"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut students = Vec::new();
        for row in rows {
            let total_apps: i64 = row.get(2);
            let screenings_passed: i64 = row.get(3);
            let interviews_passed: i64 = row.get(4);

            let success_rate = if total_apps > 0 {
                (interviews_passed as f64 / total_apps as f64) * 100.0
            } else {
                0.0
            };

            students.push(StudentPerformance {
                student_email: row.get(0),
                student_name: row.get(1),
                total_applications: total_apps,
                screenings_passed,
                interviews_passed,
                success_rate,
            });
        }

        Ok(students)
    }
}