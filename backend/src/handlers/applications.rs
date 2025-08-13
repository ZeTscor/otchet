use axum::{
    extract::{Extension, Multipart, Path, State},
    http::StatusCode,
    response::Json,
};
use infer;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::auth::AuthUser,
    models::{
        application::{
            Application, ApplicationResponse, CreateApplicationRequest, UpdateApplicationRequest,
        },
        interview::{Interview, InterviewResponse, UpdateInterviewRequest},
        screening::{Screening, ScreeningResponse, UpdateScreeningRequest},
    },
    utils::errors::AppError,
    AppState,
};

const ALLOWED_EXTENSIONS: &[&str] = &[
    "mp3", "wav", "ogg", "m4a", "aac", // Audio formats
    "mp4", "webm", "mov", "avi", "mkv", // Video formats
];

const ALLOWED_MIME_TYPES: &[&str] = &[
    "audio/mpeg",
    "audio/wav",
    "audio/ogg",
    "audio/mp4",
    "audio/webm",
    "audio/aac",
    "video/mp4",
    "video/webm",
    "video/quicktime",
    "video/x-msvideo",
    "video/avi",
];

fn get_max_file_size() -> usize {
    env::var("MAX_UPLOAD_MB")
        .unwrap_or_else(|_| "500".to_string())
        .parse::<usize>()
        .unwrap_or(500)
        * 1024
        * 1024
}

fn validate_file_security(filename: &str, data: &[u8]) -> Result<String, StatusCode> {
    // Validate file size
    if data.len() > get_max_file_size() {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    // Validate file extension
    let extension = std::path::Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_lowercase();

    if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
        return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    // Validate magic bytes using infer crate
    let kind = infer::get(data).ok_or(StatusCode::UNSUPPORTED_MEDIA_TYPE)?;

    if !ALLOWED_MIME_TYPES.contains(&kind.mime_type()) {
        return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    Ok(extension)
}

pub async fn get_applications(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<ApplicationResponse>>, StatusCode> {
    // For simplicity, use separate queries to avoid complex JOIN handling
    let applications = sqlx::query_as::<_, Application>(
        "SELECT * FROM applications WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(auth_user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get all screenings for these applications in one query
    let app_ids: Vec<i32> = applications.iter().map(|a| a.id).collect();
    let screenings = if !app_ids.is_empty() {
        sqlx::query_as::<_, Screening>("SELECT * FROM screenings WHERE application_id = ANY($1)")
            .bind(&app_ids)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    // Get all interviews for these applications in one query
    let interviews = if !app_ids.is_empty() {
        sqlx::query_as::<_, Interview>("SELECT * FROM interviews WHERE application_id = ANY($1)")
            .bind(&app_ids)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    // Create hashmaps for efficient lookup
    let screening_map: std::collections::HashMap<i32, Screening> = screenings
        .into_iter()
        .map(|s| (s.application_id, s))
        .collect();
    let interview_map: std::collections::HashMap<i32, Interview> = interviews
        .into_iter()
        .map(|i| (i.application_id, i))
        .collect();

    let mut responses = Vec::new();

    for app in applications {
        let mut response = ApplicationResponse::from(app.clone());

        // Add screening if exists
        if let Some(screening) = screening_map.get(&app.id) {
            response.screening = Some(ScreeningResponse::from(screening.clone()));
        }

        // Add interview if exists
        if let Some(interview) = interview_map.get(&app.id) {
            response.interview = Some(InterviewResponse::from(interview.clone()));
        }

        responses.push(response);
    }

    Ok(Json(responses))
}

pub async fn get_application(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> Result<Json<ApplicationResponse>, StatusCode> {
    let application = sqlx::query_as::<_, Application>(
        "SELECT * FROM applications WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(auth_user.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    let mut response = ApplicationResponse::from(application.clone());

    // Fetch screening if exists
    if let Ok(screening) =
        sqlx::query_as::<_, Screening>("SELECT * FROM screenings WHERE application_id = $1")
            .bind(application.id)
            .fetch_one(&state.db)
            .await
    {
        response.screening = Some(ScreeningResponse::from(screening));
    }

    // Fetch interview if exists
    if let Ok(interview) =
        sqlx::query_as::<_, Interview>("SELECT * FROM interviews WHERE application_id = $1")
            .bind(application.id)
            .fetch_one(&state.db)
            .await
    {
        response.interview = Some(InterviewResponse::from(interview));
    }

    Ok(Json(response))
}

pub async fn create_application(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<CreateApplicationRequest>,
) -> Result<Json<ApplicationResponse>, AppError> {
    payload.validate()?;

    let application = sqlx::query_as::<_, Application>(
        r#"
        INSERT INTO applications (user_id, company, job_url, applied_date)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(auth_user.user_id)
    .bind(&payload.company)
    .bind(&payload.job_url)
    .bind(payload.applied_date)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApplicationResponse::from(application)))
}

pub async fn update_application(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateApplicationRequest>,
) -> Result<Json<ApplicationResponse>, StatusCode> {
    if let Err(_) = payload.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Build the query dynamically

    if payload.company.is_some()
        || payload.job_url.is_some()
        || payload.applied_date.is_some()
        || payload.status.is_some()
    {
        // For simplicity, let's update the fields that are provided
        let query = r#"
            UPDATE applications 
            SET company = COALESCE($1, company),
                job_url = COALESCE($2, job_url), 
                applied_date = COALESCE($3, applied_date),
                status = COALESCE($4, status),
                updated_at = NOW()
            WHERE id = $5 AND user_id = $6
            RETURNING *
        "#;

        let application = sqlx::query_as::<_, Application>(query)
            .bind(&payload.company)
            .bind(&payload.job_url)
            .bind(payload.applied_date)
            .bind(&payload.status)
            .bind(id)
            .bind(auth_user.user_id)
            .fetch_one(&state.db)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;

        return Ok(Json(ApplicationResponse::from(application)));
    }

    return Err(StatusCode::BAD_REQUEST);
}

pub async fn delete_application(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM applications WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn upload_screening(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
    mut multipart: Multipart,
) -> Result<Json<ScreeningResponse>, StatusCode> {
    // Check if application exists and belongs to user
    let _application = sqlx::query_as::<_, Application>(
        "SELECT * FROM applications WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(auth_user.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    let mut file_data: Option<Vec<u8>> = None;
    let mut original_filename: Option<String> = None;
    let mut screening_request = UpdateScreeningRequest {
        screening_date: None,
        result: None,
    };

    // Process multipart fields
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                let filename = field
                    .file_name()
                    .ok_or(StatusCode::BAD_REQUEST)?
                    .to_string();

                let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

                // Validate file security (extension, MIME, magic bytes)
                validate_file_security(&filename, &data)?;

                file_data = Some(data.to_vec());
                original_filename = Some(filename);
            }
            "screening_date" => {
                let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                let date_str =
                    String::from_utf8(data.to_vec()).map_err(|_| StatusCode::BAD_REQUEST)?;
                screening_request.screening_date =
                    chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok();
            }
            "screening_status" => {
                let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                let result_str =
                    String::from_utf8(data.to_vec()).map_err(|_| StatusCode::BAD_REQUEST)?;
                screening_request.result = match result_str.as_str() {
                    "passed" => Some(crate::models::screening::ScreeningResult::Passed),
                    "failed" => Some(crate::models::screening::ScreeningResult::Failed),
                    _ => None,
                };
            }
            _ => {}
        }
    }

    // Start database transaction
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut final_file_path: Option<String> = None;

    // Handle file upload if present
    if let (Some(data), Some(filename)) = (file_data, original_filename) {
        let extension = validate_file_security(&filename, &data)?;
        let unique_filename = format!("{}.{}", Uuid::new_v4(), extension);
        let temp_filename = format!("{}.tmp", unique_filename);

        let upload_dir = PathBuf::from(&state.upload_dir);
        let temp_path = upload_dir.join(&temp_filename);
        let _final_path = upload_dir.join(&unique_filename);

        // Write to temporary file first
        fs::write(&temp_path, data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        final_file_path = Some(unique_filename);
    }

    let screening_result = screening_request.result.clone();

    // Insert or update screening in transaction
    let screening = if final_file_path.is_some() {
        // File was uploaded, update everything including file_path
        sqlx::query_as::<_, Screening>(
            r#"
            INSERT INTO screenings (application_id, file_path, screening_date, result)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (application_id) 
            DO UPDATE SET 
                file_path = $2,
                screening_date = COALESCE($3, screenings.screening_date),
                result = COALESCE($4, screenings.result),
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&final_file_path)
        .bind(screening_request.screening_date)
        .bind(screening_request.result)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        // No file uploaded, only update metadata
        sqlx::query_as::<_, Screening>(
            r#"
            INSERT INTO screenings (application_id, screening_date, result)
            VALUES ($1, $2, $3)
            ON CONFLICT (application_id) 
            DO UPDATE SET 
                screening_date = COALESCE($2, screenings.screening_date),
                result = COALESCE($3, screenings.result),
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(screening_request.screening_date)
        .bind(screening_request.result)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    // Update application status if screening failed
    if let Some(ref result) = screening_result {
        if matches!(result, crate::models::screening::ScreeningResult::Failed) {
            sqlx::query("UPDATE applications SET status = 'rejected' WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    // Commit transaction
    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Move temp file to final location after successful commit
    if let Some(ref unique_filename) = final_file_path {
        let upload_dir = PathBuf::from(&state.upload_dir);
        let temp_filename = format!("{}.tmp", unique_filename);
        let temp_path = upload_dir.join(&temp_filename);
        let final_path = upload_dir.join(unique_filename);

        if let Err(_) = fs::rename(&temp_path, &final_path).await {
            // If rename fails, try to clean up temp file
            let _ = fs::remove_file(&temp_path).await;
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok(Json(ScreeningResponse::from(screening)))
}

pub async fn upload_interview(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
    mut multipart: Multipart,
) -> Result<Json<InterviewResponse>, StatusCode> {
    // Check if application exists and belongs to user
    let _application = sqlx::query_as::<_, Application>(
        "SELECT * FROM applications WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(auth_user.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    let mut file_data: Option<Vec<u8>> = None;
    let mut original_filename: Option<String> = None;
    let mut interview_request = UpdateInterviewRequest {
        interview_date: None,
        result: None,
    };

    // Process multipart fields
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                let filename = field
                    .file_name()
                    .ok_or(StatusCode::BAD_REQUEST)?
                    .to_string();

                let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

                // Validate file security (extension, MIME, magic bytes)
                validate_file_security(&filename, &data)?;

                file_data = Some(data.to_vec());
                original_filename = Some(filename);
            }
            "interview_date" => {
                let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                let date_str =
                    String::from_utf8(data.to_vec()).map_err(|_| StatusCode::BAD_REQUEST)?;
                interview_request.interview_date =
                    chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok();
            }
            "interview_status" => {
                let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                let result_str =
                    String::from_utf8(data.to_vec()).map_err(|_| StatusCode::BAD_REQUEST)?;
                interview_request.result = match result_str.as_str() {
                    "passed" => Some(crate::models::interview::InterviewResult::Passed),
                    "failed" => Some(crate::models::interview::InterviewResult::Failed),
                    _ => None,
                };
            }
            _ => {}
        }
    }

    // Start database transaction
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut final_file_path: Option<String> = None;

    // Handle file upload if present
    if let (Some(data), Some(filename)) = (file_data, original_filename) {
        let extension = validate_file_security(&filename, &data)?;
        let unique_filename = format!("{}.{}", Uuid::new_v4(), extension);
        let temp_filename = format!("{}.tmp", unique_filename);

        let upload_dir = PathBuf::from(&state.upload_dir);
        let temp_path = upload_dir.join(&temp_filename);

        // Write to temporary file first
        fs::write(&temp_path, data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        final_file_path = Some(unique_filename);
    }

    let interview_result = interview_request.result.clone();

    // Insert or update interview in transaction
    let interview = if final_file_path.is_some() {
        // File was uploaded, update everything including file_path
        sqlx::query_as::<_, Interview>(
            r#"
            INSERT INTO interviews (application_id, file_path, interview_date, result)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (application_id) 
            DO UPDATE SET 
                file_path = $2,
                interview_date = COALESCE($3, interviews.interview_date),
                result = COALESCE($4, interviews.result),
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&final_file_path)
        .bind(interview_request.interview_date)
        .bind(interview_request.result)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        // No file uploaded, only update metadata
        sqlx::query_as::<_, Interview>(
            r#"
            INSERT INTO interviews (application_id, interview_date, result)
            VALUES ($1, $2, $3)
            ON CONFLICT (application_id) 
            DO UPDATE SET 
                interview_date = COALESCE($2, interviews.interview_date),
                result = COALESCE($3, interviews.result),
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(interview_request.interview_date)
        .bind(interview_request.result)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    // Update application status based on interview result
    if let Some(ref result) = interview_result {
        let new_status = match result {
            crate::models::interview::InterviewResult::Passed => "next_stage",
            crate::models::interview::InterviewResult::Failed => "rejected",
        };

        sqlx::query("UPDATE applications SET status = $1 WHERE id = $2")
            .bind(new_status)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Commit transaction
    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Move temp file to final location after successful commit
    if let Some(ref unique_filename) = final_file_path {
        let upload_dir = PathBuf::from(&state.upload_dir);
        let temp_filename = format!("{}.tmp", unique_filename);
        let temp_path = upload_dir.join(&temp_filename);
        let final_path = upload_dir.join(unique_filename);

        if let Err(_) = fs::rename(&temp_path, &final_path).await {
            // If rename fails, try to clean up temp file
            let _ = fs::remove_file(&temp_path).await;
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok(Json(InterviewResponse::from(interview)))
}

pub async fn get_user_activity(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<crate::services::activity::ActivityData>>, StatusCode> {
    use crate::services::activity::{ActivityError, ActivityService};
    use crate::utils::logger::LOGGER;

    LOGGER.log_request(
        "GET",
        "/applications/activity",
        Some(auth_user.user_id),
        200,
    );

    let activity_service = ActivityService::new(state.db.clone());

    match activity_service.get_user_activity(auth_user.user_id).await {
        Ok(activity_data) => {
            LOGGER.log_business_event(
                "user_activity_request_completed",
                Some(auth_user.user_id),
                [(
                    "activity_days".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(activity_data.len())),
                )]
                .iter()
                .cloned()
                .collect(),
            );
            Ok(Json(activity_data))
        }
        Err(ActivityError::DatabaseError(msg)) => {
            let mut context = HashMap::new();
            context.insert(
                "user_id".to_string(),
                serde_json::Value::Number(serde_json::Number::from(auth_user.user_id)),
            );
            context.insert(
                "error_type".to_string(),
                serde_json::Value::String("database".to_string()),
            );
            LOGGER.log_error(&msg, context);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(ActivityError::PermissionDenied) => Err(StatusCode::FORBIDDEN),
    }
}
