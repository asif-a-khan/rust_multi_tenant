use axum::{Router, middleware};
use dotenv::dotenv;
use rust_multi_tenant::{
    database::{connect_to_master_database, run_master_migrations},
    middlewares::{auth_middleware, create_cors_layer},
    multi_tenancy::TenantConnectionManager,
    routes::{auth_routes, tenant_routes, user_routes},
    types::config::AppConfig,
    types::shared::AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Load configuration
    let config = AppConfig::from_env()?;

    // Initialize tenant manager
    let tenant_manager = TenantConnectionManager::new(config.database_config.clone()).await?;

    // Run master migrations
    let master_db = connect_to_master_database(&config.database_config).await?;
    run_master_migrations(&master_db).await?;

    let state = AppState {
        tenant_manager,
        jwt_secret: config.jwt_secret,
    };

    // Create CORS layer
    let cors = create_cors_layer();

    let app = Router::new()
        .merge(auth_routes())
        .merge(user_routes())
        .merge(tenant_routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("🚀 Multi-tenant API server running on http://0.0.0.0:8000");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
