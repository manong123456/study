//! HTTP request handlers for user authentication and management.
//!
//! Handles registration, login, JWT issuance, and user status queries.

use std::sync::Arc;
use axum::{extract::State, Json};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use capeos_common::models::ApiResponse;
use capeos_common::error::{AppError, AppResult};
use crate::AppState;

/// Request body for user registration.
#[derive(Deserialize)]
pub struct RegisterRequest {
    /// Desired username (must be unique).
    pub username: String,
    /// Plain-text password (hashed with Argon2 before storage).
    pub password: String,
}

/// Request body for user login.
#[derive(Deserialize)]
pub struct LoginRequest {
    /// Username.
    pub username: String,
    /// Plain-text password.
    pub password: String,
}

/// Response body returned on successful login.
#[derive(Serialize)]
pub struct LoginResponse {
    /// JWT bearer token for authenticated requests.
    pub token: String,
}

/// User information returned in API responses.
#[derive(Serialize)]
pub struct UserInfo {
    /// Database user ID.
    pub id: i64,
    /// Username.
    pub username: String,
    /// Role (e.g. "user", "admin").
    pub role: String,
}

/// Indicates whether the system has been initialized with at least one user.
#[derive(Serialize)]
pub struct UserStatus {
    /// True if at least one user exists.
    pub initialized: bool,
}

/// POST `/v1/user_service/users/register` -Registers a new user.
///
/// Request body: `RegisterRequest`. Returns `UserInfo` on success.
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<ApiResponse<UserInfo>>> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .to_string();

    let username = req.username.clone();
    let user = state.db.call(move |conn| -> Result<UserInfo, rusqlite::Error> {
        conn.execute(
            "INSERT INTO users (username, password_hash) VALUES (?1, ?2)",
            rusqlite::params![username.clone(), hash],
        )?;
        let id = conn.last_insert_rowid();
        Ok(UserInfo { id, username, role: "user".to_string() })
    }).await.map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::created(user)))
}

/// POST `/v1/user_service/users/login` -Authenticates user and returns JWT.
///
/// Request body: `LoginRequest`. Returns `LoginResponse` with token on success.
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<ApiResponse<LoginResponse>>> {
    let username = req.username.clone();
    let password = req.password.clone();

    let (id, stored_hash): (i64, String) = state.db.call(move |conn| -> Result<(i64, String), rusqlite::Error> {
        conn.query_row(
            "SELECT id, password_hash FROM users WHERE username = ?1",
            [&username],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
    }).await.map_err(|_| AppError::Unauthorized("invalid credentials".to_string()))?;

    let parsed_hash = PasswordHash::new(&stored_hash)
        .map_err(|_| AppError::Internal("hash parse error".to_string()))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("invalid credentials".to_string()))?;

    let token = state.jwt.issue_token(&req.username, id)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::ok(LoginResponse { token })))
}

/// GET `/v1/user_service/users/current` -Returns the current authenticated user.
///
/// Response: `UserInfo` (TODO: extract from JWT claims).
pub async fn current_user() -> Json<ApiResponse<()>> {
    // TODO: extract from JWT claims
    Json(ApiResponse::ok_empty())
}

/// PUT `/v1/user_service/users/current/password` -Changes the current user's password.
///
/// Response: empty success (TODO: implement).
pub async fn change_password() -> Json<ApiResponse<()>> {
    // TODO: implement
    Json(ApiResponse::ok_empty())
}

/// DELETE `/v1/user_service/users/current` -Deletes the current user account.
///
/// Response: empty success (TODO: implement).
pub async fn delete_user() -> Json<ApiResponse<()>> {
    // TODO: implement
    Json(ApiResponse::ok_empty())
}

/// GET `/v1/user_service/users/status` -Returns whether the system has been initialized.
///
/// Response: `UserStatus` with `initialized` true if at least one user exists.
pub async fn user_status(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<UserStatus>> {
    let count: i64 = state.db.call(|conn| -> Result<i64, rusqlite::Error> {
        conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
    }).await.unwrap_or(0);
    Json(ApiResponse::ok(UserStatus { initialized: count > 0 }))
}

/// GET `/.well-known/jwks.json` -Returns the JSON Web Key Set for JWT verification.
///
/// Response: raw JSON string of the JWKS.
pub async fn jwks(State(state): State<Arc<AppState>>) -> String {
    state.jwt.jwks_json().to_string()
}
