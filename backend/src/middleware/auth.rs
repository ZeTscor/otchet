use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use crate::{utils::jwt::verify_jwt, AppState};

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: i32,
    pub role: String,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix

    let claims = verify_jwt(token, &state.jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let auth_user = AuthUser {
        user_id: claims.sub,
        role: claims.role,
    };

    request.extensions_mut().insert(auth_user);
    Ok(next.run(request).await)
}