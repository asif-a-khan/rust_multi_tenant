use axum::{routing::get, Router};
use crate::controllers::users::{users_index, users_create, users_update, users_delete, users_count};
use crate::types::shared::AppState;

// Create user routes with single endpoint pattern
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/users", 
            get(users_index)
            .post(users_create)
            .patch(users_update)
            .delete(users_delete)
        )
        .route("/api/users/count", get(users_count))
} 