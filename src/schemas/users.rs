use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct RegisterUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UserResponse {
    pub name: String,
    pub email: String,
}
