use sea_orm::{DatabaseConnection, Statement, DatabaseBackend, ConnectionTrait};
use chrono::{Utc, NaiveDateTime};
use uuid::Uuid;
use crate::types::shared::{CreateUserRequest, UserResponse};

pub struct TenantService {
    db: DatabaseConnection,
}

impl TenantService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
    
    pub async fn create_user(&self, user_data: CreateUserRequest) -> Result<UserResponse, sea_orm::DbErr> {
        let user_id = Uuid::new_v4().to_string();
        let now = Utc::now().naive_utc();
        
        // Insert user into tenant database
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO users (id, email, first_name, last_name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
            vec![
                user_id.clone().into(),
                user_data.email.clone().into(),
                user_data.first_name.clone().into(),
                user_data.last_name.clone().into(),
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
    
    pub async fn get_users(&self) -> Result<Vec<UserResponse>, sea_orm::DbErr> {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT id, email, first_name, last_name, created_at, updated_at FROM users",
            vec![]
        );
        
        let result = self.db.query_all(stmt).await?;
        
        let mut users = Vec::new();
        for row in result {
            users.push(UserResponse {
                id: row.try_get::<String>("", "id").map_err(|_| sea_orm::DbErr::Custom("Failed to get id".to_string()))?,
                email: row.try_get::<String>("", "email").map_err(|_| sea_orm::DbErr::Custom("Failed to get email".to_string()))?,
                first_name: row.try_get::<String>("", "first_name").map_err(|_| sea_orm::DbErr::Custom("Failed to get first_name".to_string()))?,
                last_name: row.try_get::<String>("", "last_name").map_err(|_| sea_orm::DbErr::Custom("Failed to get last_name".to_string()))?,
                created_at: row.try_get::<NaiveDateTime>("", "created_at").map_err(|_| sea_orm::DbErr::Custom("Failed to get created_at".to_string()))?,
                updated_at: row.try_get::<NaiveDateTime>("", "updated_at").map_err(|_| sea_orm::DbErr::Custom("Failed to get updated_at".to_string()))?,
            });
        }
        
        Ok(users)
    }
    
    pub async fn get_user(&self, user_id: &str) -> Result<Option<UserResponse>, sea_orm::DbErr> {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT id, email, first_name, last_name, created_at, updated_at FROM users WHERE id = $1",
            vec![user_id.into()]
        );
        
        let result = self.db.query_one(stmt).await?;
        
        if let Some(row) = result {
            Ok(Some(UserResponse {
                id: row.try_get::<String>("", "id").map_err(|_| sea_orm::DbErr::Custom("Failed to get id".to_string()))?,
                email: row.try_get::<String>("", "email").map_err(|_| sea_orm::DbErr::Custom("Failed to get email".to_string()))?,
                first_name: row.try_get::<String>("", "first_name").map_err(|_| sea_orm::DbErr::Custom("Failed to get first_name".to_string()))?,
                last_name: row.try_get::<String>("", "last_name").map_err(|_| sea_orm::DbErr::Custom("Failed to get last_name".to_string()))?,
                created_at: row.try_get::<NaiveDateTime>("", "created_at").map_err(|_| sea_orm::DbErr::Custom("Failed to get created_at".to_string()))?,
                updated_at: row.try_get::<NaiveDateTime>("", "updated_at").map_err(|_| sea_orm::DbErr::Custom("Failed to get updated_at".to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn update_user(&self, user_id: &str, user_data: CreateUserRequest) -> Result<Option<UserResponse>, sea_orm::DbErr> {
        let now = Utc::now().naive_utc();
        
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE users SET email = $1, first_name = $2, last_name = $3, updated_at = $4 WHERE id = $5",
            vec![
                user_data.email.clone().into(),
                user_data.first_name.clone().into(),
                user_data.last_name.clone().into(),
                now.into(),
                user_id.into()
            ]
        );
        
        let result = self.db.execute(stmt).await?;
        
        if result.rows_affected() > 0 {
            Ok(Some(UserResponse {
                id: user_id.to_string(),
                email: user_data.email,
                first_name: user_data.first_name,
                last_name: user_data.last_name,
                created_at: Utc::now().naive_utc(), // Would get from database
                updated_at: now,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn delete_user(&self, user_id: &str) -> Result<bool, sea_orm::DbErr> {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "DELETE FROM users WHERE id = $1",
            vec![user_id.into()]
        );
        
        let result = self.db.execute(stmt).await?;
        
        Ok(result.rows_affected() > 0)
    }
} 