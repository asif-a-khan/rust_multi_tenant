use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, Deserialize)]
pub struct UsersUrlParams {
    pub id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsersCountUrlParams {
    pub tenant_id: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsersRequestBody {
    pub id: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub enum UsersResponseType {
    SingleUser(UserResponse),
    MultipleUsers(Vec<UserResponse>),
    PaginatedUsers {
        users: Vec<UserResponse>,
        total_count: u64,
        page: u32,
        page_size: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub tenant_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
} 