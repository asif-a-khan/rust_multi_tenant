use axum::{routing::post, Router};
use crate::controllers::auth::{login, register, create_tenant};
use crate::types::shared::AppState;

// Create auth routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/tenants", post(create_tenant))
} 