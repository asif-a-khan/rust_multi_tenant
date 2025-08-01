use sea_orm_migration::prelude::*;

pub struct TenantMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TenantMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_users_table::Migration),
            Box::new(m20240101_000002_create_products_table::Migration),
            Box::new(m20240101_000003_create_orders_table::Migration),
        ]
    }
}

pub mod m20240101_000001_create_users_table;
pub mod m20240101_000002_create_products_table;
pub mod m20240101_000003_create_orders_table; 