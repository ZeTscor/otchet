use axum::{
    extract::{Path, Extension, Query, State},
    http::{StatusCode, header, HeaderMap},
    response::Response,
    body::Body,
};
use tokio::fs;
use std::path::PathBuf;
use serde::Deserialize;

use crate::{middleware::auth::AuthUser, AppState, utils::jwt::verify_jwt};
use sqlx::PgPool;

pub async fn serve_file(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    // Check if user is admin (can access all files) or if user owns the file
    let can_access = if auth_user.role == "admin" {
        true
    } else {
        // For students, check if they own the file
        check_file_ownership(&state.db, &filename, auth_user.user_id).await?
    };
    
    if !can_access {
        return Err(StatusCode::FORBIDDEN);
    }
    let file_path = PathBuf::from("uploads").join(&filename);
    
    // Security check: ensure the path is within uploads directory
    if !file_path.starts_with("uploads") {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check if file exists
    if !file_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Read file
    let file_content = fs::read(&file_path).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Determine content type based on file extension
    let content_type = match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("pdf") => "application/pdf",
        _ => "application/octet-stream",
    };

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
    headers.insert(header::CONTENT_LENGTH, file_content.len().to_string().parse().unwrap());

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, file_content.len())
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
    // Verify token
    let claims = verify_jwt(&params.token, &state.jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
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
    
    let file_path = PathBuf::from("uploads").join(&filename);
    
    // Security check: ensure the path is within uploads directory
    if !file_path.starts_with("uploads") {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check if file exists
    if !file_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Read file
    let file_content = fs::read(&file_path).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Determine content type based on file extension
    let content_type = match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("pdf") => "application/pdf",
        _ => "application/octet-stream",
    };

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
    headers.insert(header::CONTENT_LENGTH, file_content.len().to_string().parse().unwrap());

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, file_content.len())
        .body(Body::from(file_content))
        .unwrap())
}

async fn check_file_ownership(db: &PgPool, filename: &str, user_id: i32) -> Result<bool, StatusCode> {
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