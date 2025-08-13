use crate::{models::user::UserRole, utils::jwt::verify_jwt, AppState};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: i32,
    pub role: UserRole,
}

impl AuthUser {
    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin)
    }

    pub fn is_student(&self) -> bool {
        matches!(self.role, UserRole::Student)
    }

    pub fn role_str(&self) -> &'static str {
        match self.role {
            UserRole::Admin => "admin",
            UserRole::Student => "student",
        }
    }
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

    let claims = verify_jwt(token, &state.jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let role = match claims.role.as_str() {
        "admin" => UserRole::Admin,
        "student" => UserRole::Student,
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let auth_user = AuthUser {
        user_id: claims.sub,
        role,
    };

    request.extensions_mut().insert(auth_user);
    Ok(next.run(request).await)
}
