use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{Extension, State},
    response::Json,
};
use bcrypt::verify;
use password_hash::{rand_core::OsRng, SaltString};
use std::env;
use validator::Validate;

use crate::{
    middleware::auth::AuthUser,
    models::user::{CreateUserRequest, LoginRequest, LoginResponse, User, UserResponse, UserRole},
    utils::{errors::AppError, jwt::create_jwt},
    AppState,
};

fn validate_password_length(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters long".to_string(),
        ));
    }
    if password.len() > 72 {
        return Err(AppError::BadRequest(
            "Password too long (max 72 characters for compatibility)".to_string(),
        ));
    }
    Ok(())
}

fn hash_password_argon2(password: &str) -> Result<String, AppError> {
    validate_password_length(password)?;

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AppError::InternalServerError("Failed to hash password".to_string()))?
        .to_string()
        .parse()
        .map_err(|_| AppError::InternalServerError("Failed to format password hash".to_string()))
}

async fn verify_password_and_rehash(
    password: &str,
    stored_hash: &str,
    user_id: i32,
    db: &sqlx::PgPool,
) -> Result<bool, AppError> {
    // First try Argon2
    if let Ok(parsed_hash) = PasswordHash::new(stored_hash) {
        if parsed_hash.algorithm.as_str().starts_with("argon2") {
            let argon2 = Argon2::default();
            return Ok(argon2
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok());
        }
    }

    // Fallback to bcrypt
    let is_valid_bcrypt = verify(password, stored_hash)
        .map_err(|_| AppError::InternalServerError("Failed to verify password".to_string()))?;

    if is_valid_bcrypt {
        // Rehash with Argon2 and update in database
        if let Ok(new_hash) = hash_password_argon2(password) {
            let _ = sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
                .bind(&new_hash)
                .bind(user_id)
                .execute(db)
                .await;
        }
    }

    Ok(is_valid_bcrypt)
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    payload.validate()?;

    let password_hash = hash_password_argon2(&payload.password)?;

    // Check admin code to determine role from environment variable
    let admin_code = env::var("ADMIN_CODE").map_err(|_| {
        AppError::InternalServerError("ADMIN_CODE environment variable not set".to_string())
    })?;

    let role = if let Some(code) = &payload.admin_code {
        if !code.is_empty() {
            if code == &admin_code {
                UserRole::Admin
            } else {
                return Err(AppError::BadRequest("Invalid admin code".to_string()));
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
    if !auth_user.is_admin() {
        return Err(AppError::Forbidden(
            "Only admins can create new admins".to_string(),
        ));
    }

    payload.validate()?;

    let password_hash = hash_password_argon2(&payload.password)?;

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

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_one(&state.db)
        .await
        .map_err(|_| AppError::Unauthorized("Invalid email or password".to_string()))?;

    let is_valid =
        verify_password_and_rehash(&payload.password, &user.password_hash, user.id, &state.db)
            .await?;

    if !is_valid {
        return Err(AppError::Unauthorized(
            "Invalid email or password".to_string(),
        ));
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
