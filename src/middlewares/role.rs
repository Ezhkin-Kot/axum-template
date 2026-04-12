use std::task::{Context, Poll};

use axum::response::IntoResponse;
use http::Request;
use tower::{Layer, Service};

use crate::{errors::auth::AuthError, models::users::Role, services::auth::tokens::Claims};

#[derive(Clone)]
pub struct RoleLayer {
    allowed_roles: Vec<Role>,
}

impl<S> Layer<S> for RoleLayer {
    type Service = RoleGuardMiddleware<S>;
    fn layer(&self, inner: S) -> Self::Service {
        RoleGuardMiddleware {
            inner,
            allowed_roles: self.allowed_roles.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RoleGuardMiddleware<S> {
    inner: S,
    allowed_roles: Vec<Role>,
}

impl<S, ReqBody> Service<Request<ReqBody>> for RoleGuardMiddleware<S>
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

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let claims = req.extensions().get::<Claims>().cloned();
        let allowed_roles = self.allowed_roles.clone();
        Box::pin(async move {
            let claims = match claims {
                Some(cl) => cl,
                None => return Err(AuthError::Unauthorized.into_response()),
            };

            if !allowed_roles.contains(&Role::from(claims.role)) {
                return Err(AuthError::Forbidden.into_response());
            }
            inner.call(req).await
        })
    }
}
