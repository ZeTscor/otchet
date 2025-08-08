use axum::{
    extract::{State, Extension},
    response::Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use validator::Validate;

use crate::{
    models::user::{CreateUserRequest, LoginRequest, LoginResponse, User, UserResponse, UserRole},
    middleware::auth::AuthUser,
    utils::{jwt::create_jwt, errors::AppError},
    AppState,
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    payload.validate()?;

    let password_hash = hash(&payload.password, DEFAULT_COST)
        .map_err(|_| AppError::InternalServerError("Failed to hash password".to_string()))?;

    // Check admin code to determine role
    const ADMIN_CODE: &str = "ADMIN2025"; // Можно вынести в переменную окружения
    let role = if let Some(code) = &payload.admin_code {
        if !code.is_empty() {
            if code == ADMIN_CODE {
                UserRole::Admin
            } else {
                return Err(AppError::BadRequest("Неверный админский код".to_string()));
            }
        } else {
            UserRole::Student
        }
    } else {
        UserRole::Student
    };

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, password_hash, first_name, last_name, role)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&role)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(UserResponse::from(user)))
}

pub async fn register_admin(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    // Only existing admins can create new admins
    if auth_user.role != "admin" {
        return Err(AppError::Forbidden("Only admins can create new admins".to_string()));
    }

    payload.validate()?;

    let password_hash = hash(&payload.password, DEFAULT_COST)
        .map_err(|_| AppError::InternalServerError("Failed to hash password".to_string()))?;

    let role = payload.role.unwrap_or(UserRole::Admin);

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, password_hash, first_name, last_name, role)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&role)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(UserResponse::from(user)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    payload.validate()?;

    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_one(&state.db)
    .await
    .map_err(|_| AppError::Unauthorized("Invalid email or password".to_string()))?;

    let is_valid = verify(&payload.password, &user.password_hash)
        .map_err(|_| AppError::InternalServerError("Failed to verify password".to_string()))?;

    if !is_valid {
        return Err(AppError::Unauthorized("Invalid email or password".to_string()));
    }

    let role_str = match user.role {
        UserRole::Student => "student",
        UserRole::Admin => "admin",
    };

    let token = create_jwt(user.id, role_str, &state.jwt_secret)
        .map_err(|_| AppError::InternalServerError("Failed to create token".to_string()))?;

    Ok(Json(LoginResponse {
        token,
        user: UserResponse::from(user),
    }))
}