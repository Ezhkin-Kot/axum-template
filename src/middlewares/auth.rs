use std::{
    sync::Arc,
    task::{Context, Poll},
};

use axum::response::IntoResponse;
use http::Request;
use sqlx::Postgres;
use tower::{Layer, Service};

use crate::{
    errors::{auth::AuthError, tokens::TokenError},
    services::auth::tokens::TokenService,
};

#[derive(Clone)]
pub struct AuthLayer {
    pub token_serv: Arc<TokenService<Postgres>>,
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;
    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware {
            inner,
            token_serv: self.token_serv.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
    token_serv: Arc<TokenService<Postgres>>,
}

impl<S, ReqBody> Service<Request<ReqBody>> for AuthMiddleware<S>
where
    S: Service<Request<ReqBody>, Error = axum::response::Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = axum::response::Response;
    type Future =
        futures::future::BoxFuture<'static, Result<Self::Response, axum::response::Response>>;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), axum::response::Response>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let token_serv = self.token_serv.clone();
        Box::pin(async move {
            let auth_header = req
                .headers()
                .get(http::header::AUTHORIZATION)
                .and_then(|header| header.to_str().ok())
                .ok_or(AuthError::Unauthorized.into_response())?;

            let (schema, token) = auth_header
                .split_once(' ')
                .ok_or(TokenError::InvalidAuthHeader.into_response())?;

            if !schema.eq_ignore_ascii_case("Bearer") {
                return Err(TokenError::InvalidAuthSchema.into_response());
            }

            let claims = match token_serv.validate_access_token(token) {
                Ok(cl) => cl,
                Err(er) => return Err(er.into_response()),
            };

            // NOTE: Если токен валидный, то извлеченные claims добавили в запрос
            req.extensions_mut().insert(claims);

            inner.call(req).await
        })
    }
}

