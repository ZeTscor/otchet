use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::{middleware::auth::AuthUser, services::notification::NotificationService, AppState};

#[derive(Debug, Deserialize)]
pub struct NotificationQuery {
    pub days: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub message: String,
    pub processed_users: usize,
    pub total_stale_applications: usize,
}

pub async fn trigger_notifications(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<NotificationQuery>,
) -> Result<Json<NotificationResponse>, StatusCode> {
    // Only admins can trigger notifications
    if !auth_user.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    let notification_service = NotificationService::new(state.db.clone());
    let days = query.days.unwrap_or(7);

    // Get stale applications for counting
    let stale_applications = notification_service
        .find_stale_applications(days)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Group by user to count unique users
    let mut unique_users = std::collections::HashSet::new();
    for app in &stale_applications {
        unique_users.insert(app.user_id);
    }

    // Process notifications
    notification_service
        .process_stale_notifications_with_days(days)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(NotificationResponse {
        message: format!(
            "Notifications processed for applications older than {} days",
            days
        ),
        processed_users: unique_users.len(),
        total_stale_applications: stale_applications.len(),
    }))
}

pub async fn get_stale_applications(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<NotificationQuery>,
) -> Result<Json<Vec<crate::models::application::ApplicationResponse>>, StatusCode> {
    // Allow both admins and students to view their own stale applications
    let notification_service = NotificationService::new(state.db.clone());
    let days = query.days.unwrap_or(7);

    let stale_applications = if auth_user.is_admin() {
        // Admin can see all stale applications
        notification_service
            .find_stale_applications(days)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        // Students can only see their own stale applications
        notification_service
            .find_user_stale_applications(auth_user.user_id, days)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let responses: Vec<crate::models::application::ApplicationResponse> = stale_applications
        .into_iter()
        .map(crate::models::application::ApplicationResponse::from)
        .collect();

    Ok(Json(responses))
}
