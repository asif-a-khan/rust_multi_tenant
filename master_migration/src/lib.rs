use sea_orm_migration::prelude::*;

pub struct MasterMigrator;

#[async_trait::async_trait]
impl MigratorTrait for MasterMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_tenants_table::Migration),
            Box::new(m20240101_000002_create_users_table::Migration),
            Box::new(m20240101_000003_create_permissions_table::Migration),
        ]
    }
}

pub mod m20240101_000001_create_tenants_table;
pub mod m20240101_000002_create_users_table;
pub mod m20240101_000003_create_permissions_table; 