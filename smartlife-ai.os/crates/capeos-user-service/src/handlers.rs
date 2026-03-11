use std::sync::Arc;
use axum::{extract::State, Json};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use capeos_common::models::ApiResponse;
use capeos_common::error::{AppError, AppResult};
use crate::AppState;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct UserStatus {
    pub initialized: bool,
}

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

pub async fn current_user() -> Json<ApiResponse<()>> {
    // TODO: extract from JWT claims
    Json(ApiResponse::ok_empty())
}

pub async fn change_password() -> Json<ApiResponse<()>> {
    // TODO: implement
    Json(ApiResponse::ok_empty())
}

pub async fn delete_user() -> Json<ApiResponse<()>> {
    // TODO: implement
    Json(ApiResponse::ok_empty())
}

pub async fn user_status(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<UserStatus>> {
    let count: i64 = state.db.call(|conn| -> Result<i64, rusqlite::Error> {
        conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
    }).await.unwrap_or(0);
    Json(ApiResponse::ok(UserStatus { initialized: count > 0 }))
}

pub async fn jwks(State(state): State<Arc<AppState>>) -> String {
    state.jwt.jwks_json().to_string()
}
