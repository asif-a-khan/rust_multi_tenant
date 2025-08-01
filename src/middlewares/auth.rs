use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use crate::{types::shared::{TenantContext, AppState}};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,           // User ID
    pub tenant_id: String,      // Tenant ID
    pub exp: usize,            // Expiration time
    pub iat: usize,            // Issued at
    pub permissions: Vec<String>, // User permissions
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract JWT token from Authorization header
    let token = extract_token_from_request(&request)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Validate and decode JWT
    let claims = validate_jwt_token(&token, &state.jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Get tenant database connection
    let db_connection = state.tenant_manager
        .get_tenant_connection(&claims.tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Create tenant context
    let tenant_context = TenantContext {
        tenant_id: claims.tenant_id,
        user_id: claims.sub,
        permissions: claims.permissions,
    };
    
    // Attach to request extensions
    request.extensions_mut().insert(tenant_context);
    request.extensions_mut().insert(db_connection);
    
    Ok(next.run(request).await)
}

fn extract_token_from_request(request: &Request) -> Option<String> {
    request.headers()
        .get("Authorization")
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_str| {
            if auth_str.starts_with("Bearer ") {
                Some(auth_str[7..].to_string())
            } else {
                None
            }
        })
}

fn validate_jwt_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);
    
    let token_data = decode::<Claims>(token, &key, &validation)?;
    Ok(token_data.claims)
}

pub fn create_jwt_token(
    user_id: &str,
    tenant_id: &str,
    permissions: &[String],
    secret: &str,
    expiration: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + chrono::Duration::seconds(expiration as i64);
    
    let claims = Claims {
        sub: user_id.to_string(),
        tenant_id: tenant_id.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
        permissions: permissions.to_vec(),
    };
    
    let key = EncodingKey::from_secret(secret.as_ref());
    encode(&Header::default(), &claims, &key)
}

pub async fn require_permission(
    tenant_context: &TenantContext,
    required_permission: &str,
) -> Result<(), StatusCode> {
    if tenant_context.permissions.contains(&required_permission.to_string()) {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
} 