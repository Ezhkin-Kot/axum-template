use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    AppState,
    services::auth::password_hashing::hash_password,
    errors::auth::AuthError,
    schemas::users::{LoginUser, RegisterUser},
};

// TODO: реализовать ошибки Auth и тп и реализовать для них IntoResponse
pub async fn register(
    State(state): State<AppState>,
    Json(user_data): Json<RegisterUser>,
) -> Result<impl IntoResponse, AuthError> {
    let pool = state.db_pool.clone();
    let existing = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(&user_data.email)
        .fetch_optional(pool.as_ref())
        .await?;

    if existing.is_some() {
        return Err(AuthError::UserAlreadyExists);
    }

    let password_hash = hash_password(&user_data.password);
    let _user = sqlx::query("INSERT INTO users (name, email, password_hash) VALUES ($1, $2, $3);")
        .bind(user_data.name)
        .bind(&user_data.email)
        .bind(password_hash)
        .execute(pool.as_ref())
        .await?;

    Ok((StatusCode::OK, Json(user_data.email)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(user_data): Json<LoginUser>,
) -> Result<impl IntoResponse, AuthError> {
    let pool = state.db_pool.clone();
    let existing = sqlx::query("SELECT id FROM users WHERE email = $1 AND password_hash = $2;")
        .bind(&user_data.email)
        .bind(hash_password(&user_data.password))
        .fetch_optional(pool.as_ref())
        .await?;

    if existing.is_some() {
        return Ok((StatusCode::OK, Json(user_data.email)));
    }

    Err(AuthError::NotFound)
}
