use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

pub async fn run_master_migrations(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    master_migration::MasterMigrator::up(db, None).await
}

pub async fn run_tenant_migrations(db_url: &str) -> Result<(), sea_orm::DbErr> {
    let db = Database::connect(db_url).await?;
    tenant_migration::TenantMigrator::up(&db, None).await
} 