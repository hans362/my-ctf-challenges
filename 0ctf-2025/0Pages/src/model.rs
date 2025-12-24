use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub status: String,
    pub username: String,
}

#[derive(Serialize)]
pub struct LogoutResponse {
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct SiteManifest {
    pub site_id: Option<String>,
    pub owner: Option<String>,
    pub webroot: String,
    pub deployed_at: Option<u64>,
}

#[derive(Serialize)]
pub struct SiteListResponse {
    pub status: String,
    pub sites: Vec<SiteManifest>,
}

#[derive(Serialize)]
pub struct SiteDeployResponse {
    pub status: String,
    pub data: SiteManifest,
}

#[derive(Serialize)]
pub struct SiteDeleteResponse {
    pub status: String,
}
