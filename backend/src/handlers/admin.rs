use axum::{
    extract::{State, Extension, Query, Path},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    middleware::auth::AuthUser,
    models::{
        application::{Application, ApplicationResponse, ApplicationStatus},
        user::{User, UserResponse},
    },
    AppState,
};

#[derive(Debug, Serialize)]
pub struct AnalyticsResponse {
    pub total_students: i64,
    pub total_applications: i64,
    pub status_breakdown: HashMap<String, i64>,
    pub company_stats: Vec<CompanyStats>,
    pub popular_job_urls: Vec<JobUrlStats>,
    pub stale_applications: Vec<ApplicationResponse>,
    pub screening_stats: ScreeningStats,
    pub interview_stats: InterviewStats,
    pub daily_stats: Vec<DailyStat>,
    pub success_rate: SuccessRateStats,
    pub response_times: ResponseTimeStats,
    pub top_performing_students: Vec<StudentPerformance>,
}

#[derive(Debug, Serialize)]
pub struct ScreeningStats {
    pub total_screenings: i64,
    pub passed: i64,
    pub failed: i64,
    pub pending: i64,
}

#[derive(Debug, Serialize)]
pub struct InterviewStats {
    pub total_interviews: i64,
    pub passed: i64,
    pub failed: i64,
    pub pending: i64,
}

#[derive(Debug, Serialize)]
pub struct DailyStat {
    pub date: String,
    pub applications_count: i64,
    pub screenings_count: i64,
    pub interviews_count: i64,
}

#[derive(Debug, Serialize)]
pub struct SuccessRateStats {
    pub overall_success_rate: f64,
    pub screening_to_interview_rate: f64,
    pub interview_success_rate: f64,
    pub applications_with_urls: i64,
    pub applications_without_urls: i64,
}

#[derive(Debug, Serialize)]
pub struct ResponseTimeStats {
    pub avg_days_to_screening: f64,
    pub avg_days_to_interview: f64,
    pub fastest_screening_days: i32,
    pub slowest_screening_days: i32,
}

#[derive(Debug, Serialize)]
pub struct StudentPerformance {
    pub student_email: String,
    pub student_name: String,
    pub total_applications: i64,
    pub screenings_passed: i64,
    pub interviews_passed: i64,
    pub success_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct CompanyStats {
    pub company: String,
    pub application_count: i64,
    pub unique_students: i64,
}

#[derive(Debug, Serialize)]
pub struct JobUrlStats {
    pub job_url: String,
    pub application_count: i64,
    pub unique_students: i64,
}

#[derive(Debug, Deserialize)]
pub struct AdminQuery {
    pub company: Option<String>,
    pub status: Option<ApplicationStatus>,
    pub days_stale: Option<i32>,
}

pub async fn get_analytics(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(_query): Query<AdminQuery>,
) -> Result<Json<AnalyticsResponse>, StatusCode> {
    use crate::utils::logger::LOGGER;
    use crate::services::analytics::{AnalyticsService, AnalyticsError};

    // Check if user is admin
    if auth_user.role != "admin" {
        LOGGER.log_business_event(
            "unauthorized_analytics_access",
            Some(auth_user.user_id),
            [("role".to_string(), serde_json::Value::String(auth_user.role))].iter().cloned().collect()
        );
        return Err(StatusCode::FORBIDDEN);
    }

    LOGGER.log_request("GET", "/admin/analytics", Some(auth_user.user_id), 200);

    let analytics_service = AnalyticsService::new(state.db.clone());
    
    match analytics_service.get_comprehensive_analytics().await {
        Ok(analytics) => {
            LOGGER.log_business_event(
                "analytics_request_completed",
                Some(auth_user.user_id),
                HashMap::new()
            );
            Ok(Json(analytics))
        }
        Err(AnalyticsError::DatabaseError(msg)) => {
            let mut context = HashMap::new();
            context.insert("user_id".to_string(), serde_json::Value::Number(
                serde_json::Number::from(auth_user.user_id)
            ));
            context.insert("error_type".to_string(), serde_json::Value::String("database".to_string()));
            LOGGER.log_error(&msg, context);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(AnalyticsError::PermissionDenied) => {
            Err(StatusCode::FORBIDDEN)
        }
    }
}

pub async fn get_all_students(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<UserResponse>>, StatusCode> {
    // Check if user is admin
    if auth_user.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    let students = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE role = 'student' ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(students.into_iter().map(UserResponse::from).collect()))
}

pub async fn get_all_applications(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(_query): Query<AdminQuery>,
) -> Result<Json<Vec<ApplicationResponse>>, StatusCode> {
    // Check if user is admin
    if auth_user.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    let applications = sqlx::query_as::<_, Application>(
        "SELECT * FROM applications ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut responses = Vec::new();
    for app in applications {
        let mut response = ApplicationResponse::from(app.clone());
        
        // Fetch screening if exists
        if let Ok(screening) = sqlx::query_as::<_, crate::models::screening::Screening>(
            "SELECT * FROM screenings WHERE application_id = $1"
        )
        .bind(app.id)
        .fetch_one(&state.db)
        .await {
            response.screening = Some(crate::models::screening::ScreeningResponse::from(screening));
        }

        // Fetch interview if exists
        if let Ok(interview) = sqlx::query_as::<_, crate::models::interview::Interview>(
            "SELECT * FROM interviews WHERE application_id = $1"
        )
        .bind(app.id)
        .fetch_one(&state.db)
        .await {
            response.interview = Some(crate::models::interview::InterviewResponse::from(interview));
        }

        responses.push(response);
    }

    Ok(Json(responses))
}

pub async fn get_admin_activity(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<crate::services::activity::ActivityData>>, StatusCode> {
    use crate::services::activity::{ActivityService, ActivityError};
    use crate::utils::logger::LOGGER;

    if auth_user.role != "admin" {
        LOGGER.log_business_event(
            "unauthorized_admin_activity_access",
            Some(auth_user.user_id),
            [("role".to_string(), serde_json::Value::String(auth_user.role))].iter().cloned().collect()
        );
        return Err(StatusCode::FORBIDDEN);
    }

    LOGGER.log_request("GET", "/admin/activity", Some(auth_user.user_id), 200);

    let activity_service = ActivityService::new(state.db.clone());
    
    match activity_service.get_admin_activity().await {
        Ok(activity_data) => {
            LOGGER.log_business_event(
                "admin_activity_request_completed",
                Some(auth_user.user_id),
                [("activity_days".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(activity_data.len())
                ))].iter().cloned().collect()
            );
            Ok(Json(activity_data))
        }
        Err(ActivityError::DatabaseError(msg)) => {
            let mut context = HashMap::new();
            context.insert("user_id".to_string(), serde_json::Value::Number(
                serde_json::Number::from(auth_user.user_id)
            ));
            context.insert("error_type".to_string(), serde_json::Value::String("database".to_string()));
            LOGGER.log_error(&msg, context);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(ActivityError::PermissionDenied) => {
            Err(StatusCode::FORBIDDEN)
        }
    }
}

pub async fn get_user_activity_admin(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<i32>,
) -> Result<Json<Vec<crate::services::activity::ActivityData>>, StatusCode> {
    use crate::services::activity::{ActivityService, ActivityError};
    use crate::utils::logger::LOGGER;

    // Only admins can access user activity
    if auth_user.role != "admin" {
        LOGGER.log_business_event(
            "unauthorized_user_activity_access",
            Some(auth_user.user_id),
            [("target_user_id".to_string(), serde_json::Value::Number(
                serde_json::Number::from(user_id)
            ))].iter().cloned().collect()
        );
        return Err(StatusCode::FORBIDDEN);
    }

    LOGGER.log_business_event(
        "user_activity_request_started",
        Some(auth_user.user_id),
        [("target_user_id".to_string(), serde_json::Value::Number(
            serde_json::Number::from(user_id)
        ))].iter().cloned().collect()
    );

    let activity_service = ActivityService::new(state.pool.clone());

    match activity_service.get_user_activity(user_id).await {
        Ok(activity_data) => {
            LOGGER.log_business_event(
                "user_activity_request_completed",
                Some(auth_user.user_id),
                [
                    ("target_user_id".to_string(), serde_json::Value::Number(
                        serde_json::Number::from(user_id)
                    )),
                    ("activity_days".to_string(), serde_json::Value::Number(
                        serde_json::Number::from(activity_data.len())
                    ))
                ].iter().cloned().collect()
            );
            Ok(Json(activity_data))
        }
        Err(ActivityError::DatabaseError(msg)) => {
            let mut context = HashMap::new();
            context.insert("admin_user_id".to_string(), serde_json::Value::Number(
                serde_json::Number::from(auth_user.user_id)
            ));
            context.insert("target_user_id".to_string(), serde_json::Value::Number(
                serde_json::Number::from(user_id)
            ));
            context.insert("error_type".to_string(), serde_json::Value::String("database".to_string()));
            LOGGER.log_error(&msg, context);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(ActivityError::PermissionDenied) => {
            Err(StatusCode::FORBIDDEN)
        }
    }
}