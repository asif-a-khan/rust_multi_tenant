use sea_orm::{DatabaseConnection, Statement, DatabaseBackend, ConnectionTrait};
use chrono::{Utc, NaiveDateTime};
use uuid::Uuid;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use password_hash::{rand_core::OsRng, SaltString};
use crate::types::shared::{CreateTenantRequest, TenantResponse, CreateUserRequest, UserResponse, LoginRequest, LoginResponse};
use crate::middlewares::create_jwt_token;

pub struct MasterService {
    db: DatabaseConnection,
}

impl MasterService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
    
    pub async fn create_tenant(&self, tenant_data: CreateTenantRequest) -> Result<TenantResponse, sea_orm::DbErr> {
        let tenant_id = tenant_data.id;
        let name = tenant_data.name;
        let now = Utc::now().naive_utc();
        
        // Insert tenant into master database
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO tenants (id, name, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
            vec![
                tenant_id.clone().into(),
                name.clone().into(),
                "active".into(),
                now.into(),
                now.into()
            ]
        );
        
        self.db.execute(stmt).await?;
        
        Ok(TenantResponse {
            id: tenant_id,
            name,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        })
    }
    
    pub async fn get_tenant(&self, tenant_id: &str) -> Result<Option<TenantResponse>, sea_orm::DbErr> {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT id, name, status, created_at, updated_at FROM tenants WHERE id = $1",
            vec![tenant_id.into()]
        );
        
        let result = self.db.query_one(stmt).await?;
        
        if let Some(row) = result {
            Ok(Some(TenantResponse {
                id: row.try_get::<String>("", "id").map_err(|_| sea_orm::DbErr::Custom("Failed to get id".to_string()))?,
                name: row.try_get::<String>("", "name").map_err(|_| sea_orm::DbErr::Custom("Failed to get name".to_string()))?,
                status: row.try_get::<String>("", "status").map_err(|_| sea_orm::DbErr::Custom("Failed to get status".to_string()))?,
                created_at: row.try_get::<NaiveDateTime>("", "created_at").map_err(|_| sea_orm::DbErr::Custom("Failed to get created_at".to_string()))?,
                updated_at: row.try_get::<NaiveDateTime>("", "updated_at").map_err(|_| sea_orm::DbErr::Custom("Failed to get updated_at".to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn create_user(&self, user_data: CreateUserRequest, tenant_id: &str) -> Result<UserResponse, sea_orm::DbErr> {
        let user_id = Uuid::new_v4().to_string();
        let password_hash = hash_password(&user_data.password)?;
        let now = Utc::now().naive_utc();
        
        // Insert user into master database
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO users (id, tenant_id, email, password_hash, permissions, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            vec![
                user_id.clone().into(),
                tenant_id.into(),
                user_data.email.clone().into(),
                password_hash.into(),
                serde_json::json!(["users:read", "users:write"]).into(),
                now.into(),
                now.into()
            ]
        );
        
        self.db.execute(stmt).await?;
        
        Ok(UserResponse {
            id: user_id,
            email: user_data.email,
            first_name: user_data.first_name,
            last_name: user_data.last_name,
            created_at: now,
            updated_at: now,
        })
    }
    
    pub async fn authenticate_user(&self, login_data: LoginRequest, tenant_id: &str) -> Result<Option<LoginResponse>, sea_orm::DbErr> {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT id, email, password_hash, permissions FROM users WHERE email = $1 AND tenant_id = $2",
            vec![login_data.email.clone().into(), tenant_id.into()]
        );
        
        let result = self.db.query_one(stmt).await?;
        
        if let Some(row) = result {
            let user_id: String = row.try_get::<String>("", "id").map_err(|_| sea_orm::DbErr::Custom("Failed to get user id".to_string()))?;
            let email: String = row.try_get::<String>("", "email").map_err(|_| sea_orm::DbErr::Custom("Failed to get email".to_string()))?;
            let password_hash: String = row.try_get::<String>("", "password_hash").map_err(|_| sea_orm::DbErr::Custom("Failed to get password_hash".to_string()))?;
            let permissions_value: serde_json::Value = row.try_get::<serde_json::Value>("", "permissions").map_err(|_| sea_orm::DbErr::Custom("Failed to get permissions".to_string()))?;
            
            if verify_password(&login_data.password, &password_hash)? {
                let permissions: Vec<String> = serde_json::from_value(permissions_value)
                    .map_err(|_| sea_orm::DbErr::Custom("Failed to parse permissions".to_string()))?;
                
                let token = create_jwt_token(
                    &user_id,
                    tenant_id,
                    &permissions,
                    "your-secret-key", // This should come from config
                    3600,
                ).map_err(|_| sea_orm::DbErr::Custom("Failed to create token".to_string()))?;
                
                Ok(Some(LoginResponse {
                    token,
                    user: UserResponse {
                        id: user_id,
                        email,
                        first_name: "".to_string(), // Would come from tenant database
                        last_name: "".to_string(),
                        created_at: Utc::now().naive_utc(), // Would come from tenant database
                        updated_at: Utc::now().naive_utc(),
                    },
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

fn hash_password(password: &str) -> Result<String, sea_orm::DbErr> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| sea_orm::DbErr::Custom(format!("Password hashing error: {}", e)))
        .map(|hash| hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> Result<bool, sea_orm::DbErr> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| sea_orm::DbErr::Custom(format!("Invalid password hash: {}", e)))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
} 