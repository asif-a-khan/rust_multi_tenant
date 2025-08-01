use axum::{
    Json,
    extract::State,
    http::StatusCode,
};
use crate::{
    types::shared::{AppState, LoginRequest, LoginResponse, CreateUserRequest, UserResponse, CreateTenantRequest, TenantResponse},
    multi_tenancy::MasterService,
};

// Auth controller functions
pub async fn login(
    State(state): State<AppState>,
    Json(login_data): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // For demo purposes, we'll use a default tenant
    let tenant_id = "demo_tenant";
    
    let master_service = MasterService::new(state.tenant_manager.get_master_connection().await);
    let login_response = master_service.authenticate_user(login_data, tenant_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    Ok(Json(login_response))
}

pub async fn register(
    State(state): State<AppState>,
    Json(user_data): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    // For demo purposes, we'll use a default tenant
    let tenant_id = "demo_tenant";
    
    let master_service = MasterService::new(state.tenant_manager.get_master_connection().await);
    let user = master_service.create_user(user_data, tenant_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(user))
}

pub async fn create_tenant(
    State(state): State<AppState>,
    Json(tenant_data): Json<CreateTenantRequest>,
) -> Result<Json<TenantResponse>, StatusCode> {
    let master_service = MasterService::new(state.tenant_manager.get_master_connection().await);
    
    // Create tenant in master database
    let tenant = master_service.create_tenant(tenant_data).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Create tenant database and run migrations
    state.tenant_manager.create_tenant_database(&tenant.id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(tenant))
} 