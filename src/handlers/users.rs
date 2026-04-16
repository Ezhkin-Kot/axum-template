use axum::{
    Extension, Json, Router, extract::State, http::StatusCode, middleware::from_fn,
    response::IntoResponse, routing::get,
};
use utoipa::OpenApi;

use crate::{
    AppState,
    errors::users::UserError,
    middlewares::{auth::auth_middleware, role::role_middleware},
    models::{tokens::Tokens, users::Role},
    repositories::users::UserRepository,
    schemas::users::{RegisterUser, UserResponse},
    services::auth::tokens::Claims,
};

pub struct UserRouter;

impl UserRouter {
    pub fn set_router(state: AppState) -> Router<AppState> {
        Router::new()
            .route("/my_profile", get(my_profile))
            .route_layer(from_fn(move |req, next| async move {
                role_middleware(req, next, Role::all()).await
            }))
            .route_layer({
                let token_serv = state.token_serv.clone();
                from_fn(move |req, next| {
                    let token_serv = token_serv.clone();
                    async move { auth_middleware(req, next, token_serv.clone()).await }
                })
            })
    }
}

#[derive(OpenApi)]
#[openapi(paths(my_profile), components(schemas(RegisterUser)))]
pub struct UserDocs;

#[utoipa::path(
    get,
    // NOTE: обязательно надо добавть, чтобы с свагера на эту ручку отправлялся токен
    security(
        ("bearer_auth" = [])
    ),
    path = "/my_profile",
    responses(
        (status = 200, description = "Пользователь зарегистрирован", body = Tokens),
        (status = 404, description = "Пользователь не найден", body = String),
        (status = 500, description = "Технические шокаладки с бд", body = String)
    )
)]
pub async fn my_profile(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, UserError> {
    let repo = state.user_repo.clone();
    // NOTE: пример получения id пользователя из claims
    match repo.get_by_id(&claims.sub).await? {
        Some(user) => Ok((StatusCode::OK, Json(UserResponse::from(user)))),
        None => Err(UserError::NotFound),
    }
}
