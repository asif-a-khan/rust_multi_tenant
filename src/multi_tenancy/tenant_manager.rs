use sea_orm::{Database, DatabaseConnection, Statement, DatabaseBackend, ConnectionTrait};
use sea_orm_migration::MigratorTrait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use crate::types::config::DatabaseConfig;

#[derive(Clone, Debug)]
pub struct TenantConnectionManager {
    connections: Arc<RwLock<HashMap<String, DatabaseConnection>>>,
    master_connection: DatabaseConnection,
    config: DatabaseConfig,
    max_connections_per_tenant: usize,
}

impl TenantConnectionManager {
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let master_connection = Database::connect(&config.master_url).await?;
        
        Ok(Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            master_connection,
            config,
            max_connections_per_tenant: 10,
        })
    }
    
    pub async fn get_tenant_connection(&self, tenant_id: &str) -> Result<DatabaseConnection> {
        let mut connections = self.connections.write().await;
        
        if let Some(conn) = connections.get(tenant_id) {
            return Ok(conn.clone());
        }
        
        // Validate tenant exists and is active
        self.validate_tenant(tenant_id).await?;
        
        // Create new connection for this tenant
        let db_url = self.build_tenant_db_url(tenant_id);
        let connection = Database::connect(&db_url).await?;
        
        // Limit connections per tenant
        if connections.len() >= self.max_connections_per_tenant {
            // Remove oldest connection (LRU could be implemented here)
            connections.clear();
        }
        
        connections.insert(tenant_id.to_string(), connection.clone());
        
        Ok(connection)
    }
    
    pub async fn get_master_connection(&self) -> DatabaseConnection {
        self.master_connection.clone()
    }
    
    async fn validate_tenant(&self, tenant_id: &str) -> Result<()> {
        // Use existing master connection to check tenant status
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT id, status FROM tenants WHERE id = $1 AND status = 'active'",
            vec![tenant_id.into()]
        );
        
        let tenant = self.master_connection.query_one(stmt).await?;
        
        if tenant.is_some() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Tenant not found or inactive"))
        }
    }
    
    fn build_tenant_db_url(&self, tenant_id: &str) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/tenant_{}",
            self.config.username,
            self.config.password,
            self.config.host,
            self.config.port,
            tenant_id
        )
    }
    
    pub async fn create_tenant_database(&self, tenant_id: &str) -> Result<()> {
        // Connect to postgres database to create new database
        let admin_db = Database::connect("postgresql://postgres@localhost/postgres").await?;
        
        // Create new database
        let db_name = format!("tenant_{}", tenant_id);
        let stmt = Statement::from_string(
            DatabaseBackend::Postgres,
            format!("CREATE DATABASE {}", db_name)
        );
        admin_db.execute(stmt).await?;
        
        // Run migrations on new database
        let tenant_db_url = self.build_tenant_db_url(tenant_id);
        self.run_tenant_migrations(&tenant_db_url).await
    }
    
    async fn run_tenant_migrations(&self, db_url: &str) -> Result<()> {
        let db = Database::connect(db_url).await?;
        tenant_migration::TenantMigrator::up(&db, None).await?;
        Ok(())
    }
} 