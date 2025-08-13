use axum::{
    body::Body,
    extract::{Extension, Path, Query, State},
    http::{header, StatusCode},
    response::Response,
};
use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs;

use crate::{middleware::auth::AuthUser, utils::jwt::verify_jwt, AppState};
use sqlx::PgPool;

pub async fn serve_file(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    // Validate filename to prevent path traversal
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check if user is admin (can access all files) or if user owns the file
    let can_access = if auth_user.is_admin() {
        true
    } else {
        // For students, check if they own the file
        check_file_ownership(&state.db, &filename, auth_user.user_id).await?
    };

    if !can_access {
        return Err(StatusCode::FORBIDDEN);
    }

    let upload_dir = PathBuf::from(&state.upload_dir);
    let file_path = upload_dir.join(&filename);

    // Security check: ensure the path is within upload directory (canonical path check)
    let canonical_file = file_path
        .canonicalize()
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let canonical_upload_dir = upload_dir
        .canonicalize()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !canonical_file.starts_with(&canonical_upload_dir) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check if file exists
    if !canonical_file.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Read file
    let file_content = fs::read(&canonical_file)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Determine content type based on file extension
    let content_type = match canonical_file.extension().and_then(|ext| ext.to_str()) {
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        Some("m4a") => "audio/mp4",
        Some("aac") => "audio/aac",
        Some("mov") => "video/quicktime",
        Some("avi") => "video/x-msvideo",
        Some("mkv") => "video/x-matroska",
        _ => "application/octet-stream",
    };

    // Create safe filename for Content-Disposition
    let safe_filename = filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect::<String>();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, file_content.len())
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", safe_filename),
        )
        .header(header::CACHE_CONTROL, "private, no-cache")
        .body(Body::from(file_content))
        .unwrap())
}

#[derive(Deserialize)]
pub struct FileQuery {
    token: String,
}

pub async fn serve_file_with_token(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Query(params): Query<FileQuery>,
) -> Result<Response<Body>, StatusCode> {
    // Validate filename to prevent path traversal
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Verify token
    let claims =
        verify_jwt(&params.token, &state.jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Check if user is admin (can access all files) or if user owns the file
    let can_access = if claims.role == "admin" {
        true
    } else {
        // For students, check if they own the file
        check_file_ownership(&state.db, &filename, claims.sub).await?
    };

    if !can_access {
        return Err(StatusCode::FORBIDDEN);
    }

    let upload_dir = PathBuf::from(&state.upload_dir);
    let file_path = upload_dir.join(&filename);

    // Security check: ensure the path is within upload directory (canonical path check)
    let canonical_file = file_path
        .canonicalize()
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let canonical_upload_dir = upload_dir
        .canonicalize()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !canonical_file.starts_with(&canonical_upload_dir) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check if file exists
    if !canonical_file.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Read file
    let file_content = fs::read(&canonical_file)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Determine content type based on file extension
    let content_type = match canonical_file.extension().and_then(|ext| ext.to_str()) {
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        Some("m4a") => "audio/mp4",
        Some("aac") => "audio/aac",
        Some("mov") => "video/quicktime",
        Some("avi") => "video/x-msvideo",
        Some("mkv") => "video/x-matroska",
        _ => "application/octet-stream",
    };

    // Create safe filename for Content-Disposition
    let safe_filename = filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect::<String>();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, file_content.len())
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", safe_filename),
        )
        .header(header::CACHE_CONTROL, "private, no-cache")
        .body(Body::from(file_content))
        .unwrap())
}

async fn check_file_ownership(
    db: &PgPool,
    filename: &str,
    user_id: i32,
) -> Result<bool, StatusCode> {
    // Check if the file belongs to any screening or interview record owned by the user
    let result = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM applications a
        LEFT JOIN screenings s ON a.id = s.application_id
        LEFT JOIN interviews i ON a.id = i.application_id
        WHERE a.user_id = $1 
        AND (s.file_path = $2 OR i.file_path = $2)
        "#,
    )
    .bind(user_id)
    .bind(filename)
    .fetch_one(db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(result > 0)
}
