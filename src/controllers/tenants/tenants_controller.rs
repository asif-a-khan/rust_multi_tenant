use axum::{
    Json,
    extract::State,
    http::StatusCode,
};
use crate::{
    types::shared::{AppState, TenantResponse},
};

// Tenants controller functions
pub async fn health_check() -> &'static str {
    "Multi-Tenant API is running!"
}

pub async fn get_tenant_info(
    State(_state): State<AppState>,
) -> Result<Json<TenantResponse>, StatusCode> {
    // This would be implemented to get current tenant info
    todo!("Implement tenant info endpoint")
} 