use sea_orm::{Database, DatabaseConnection};
use crate::types::config::DatabaseConfig;

pub async fn connect_to_master_database(config: &DatabaseConfig) -> Result<DatabaseConnection, sea_orm::DbErr> {
    Database::connect(&config.master_url).await
}

pub async fn connect_to_tenant_database(db_url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    Database::connect(db_url).await
} 