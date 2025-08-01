use sea_orm::{Database, ConnectOptions};
use sea_orm_migration::MigratorTrait;
use master_migration::MasterMigrator;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("MASTER_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost/master_db".to_string());
    
    let db = Database::connect(&database_url).await?;
    
    MasterMigrator::up(&db, None).await?;
    
    println!("Master migrations completed successfully!");
    Ok(())
} 