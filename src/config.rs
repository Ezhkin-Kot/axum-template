use sqlx::Pool;
use sqlx::Postgres;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::Arc;

use crate::repository::tokens::TokenRepo;
use crate::repository::users::UserRepo;
use crate::services::auth::tokens::TokenService;

pub struct Config {
    pub database_url: String,
    pub secret_key: String,
    pub secret_refresh_key: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect(".env not loaded"),
            secret_key: env::var("JWT_SECRET").expect(".env not loaded"),
            secret_refresh_key: env::var("JWT_SECRET_REFRESH").expect(".env not loaded"),
        }
    }
}

pub async fn get_db_pool(database_url: &str) -> sqlx::PgPool {
    PgPoolOptions::new().connect(database_url).await.unwrap()
}

#[derive(Clone)]
pub struct AppState {
    // NOTE: репозитории
    pub user_repo: Arc<UserRepo<Postgres>>,
    pub token_repo: Arc<TokenRepo<Postgres>>,

    // NOTE: сервисы
    pub token_serv: Arc<TokenService<Postgres>>,

    // NOTE: Ключи и тп
    pub secret_key: Arc<String>,
    pub secret_refresh_key: Arc<String>,
}

impl AppState {
    pub fn new(
        db_pool: Arc<Pool<Postgres>>,
        secret_key: String,
        secret_refresh_key: String,
    ) -> Self {
        let user_repo = Arc::new(UserRepo::new(db_pool.clone()));
        let token_repo = Arc::new(TokenRepo::new(db_pool.clone()));

        let secret_key = Arc::new(secret_key);
        let secret_refresh_key = Arc::new(secret_refresh_key);
        Self {
            token_serv: Arc::new(TokenService::new(
                secret_key.clone(),
                secret_refresh_key.clone(),
                5,
                token_repo.clone(),
                user_repo.clone(),
                1440,
            )),
            user_repo,
            token_repo,
            secret_key,
            secret_refresh_key,
        }
    }
}
