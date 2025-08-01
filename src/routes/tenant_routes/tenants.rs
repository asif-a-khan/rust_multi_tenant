use axum::{routing::get, Router};
use crate::controllers::tenants::health_check;
use crate::types::shared::AppState;

// Create tenant routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(health_check))
} 