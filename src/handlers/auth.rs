use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use utoipa::OpenApi;

use crate::{
    AppState,
    errors::auth::AuthError,
    schemas::users::{LoginUser, RegisterUser},
    services::auth::password_hashing::hash_password,
};

pub struct AuthRouter;

impl AuthRouter {
    pub fn set_router() -> Router<AppState> {
        Router::new()
            .route("/registration", post(register))
            .route("/login", post(login))
    }
}

#[derive(OpenApi)]
#[openapi(paths(register, login), components(schemas(LoginUser, RegisterUser)))]
pub struct AuthDocs;

// TODO: вынести логику в services

#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterUser,
    responses(
        (status = 200, description = "Пользователь зарегестрирован", body = (String, String)),
        (status = 409, description = "Пользователь с такой почтой уже существует", body = String),
        (status = 500, description = "Технические шокаладки с бд", body = String)
    )
)]
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

    Ok((
        StatusCode::OK,
        Json((user_data.email.clone(), user_data.email)),
    ))
}

#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginUser,
    responses(
        (status = 200, description = "Вход успешен", body = (String, String)),
        (status = 404, description = "Пользователь не найден", body = String),
        (status = 500, description = "Технические шокаладки с бд", body = String)
    )
)]
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
        return Ok((
            StatusCode::OK,
            Json((user_data.email.clone(), user_data.email)),
        ));
    }

    Err(AuthError::NotFound)
}
