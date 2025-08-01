use sea_orm::{Database, ConnectOptions};
use sea_orm_migration::MigratorTrait;
use tenant_migration::TenantMigrator;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable is required");
    
    let db = Database::connect(&database_url).await?;
    
    TenantMigrator::up(&db, None).await?;
    
    println!("Tenant migrations completed successfully!");
    Ok(())
} 