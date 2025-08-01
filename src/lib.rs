pub mod types;
pub mod controllers;
pub mod routes;
pub mod middlewares;
pub mod database;
pub mod multi_tenancy;
pub mod entities;

// Re-export specific items from each module to avoid conflicts
pub use types::{
    TenantContext, AppState, CreateTenantRequest, TenantResponse, 
    CreateUserRequest, LoginRequest, LoginResponse,
    UsersUrlParams, UsersCountUrlParams, UsersRequestBody, UsersResponseType, UserResponse,
    AppConfig, DatabaseConfig
};
pub use database::{connect_to_master_database, connect_to_tenant_database};
pub use multi_tenancy::{TenantConnectionManager, MasterService, TenantService};
pub use middlewares::{auth_middleware, create_jwt_token}; 