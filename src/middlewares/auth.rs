use std::{
    sync::Arc,
    task::{Context, Poll},
};

use http::Request;
use sqlx::Postgres;
use tower::{Layer, Service};

use crate::{errors::auth::AuthError, services::auth::tokens::TokenService};

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
