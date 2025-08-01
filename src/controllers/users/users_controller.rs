use axum::{Extension, Json, extract::Query, http::StatusCode, response::IntoResponse};
use uuid::Uuid;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};

use tracing::{error, info, instrument};

use crate::{
    entities::tenant::users::{Entity, Column, ActiveModel},
    types::shared::{AppState, TenantContext},
    types::users::{
        UserResponse, UsersCountUrlParams, UsersRequestBody, UsersResponseType, UsersUrlParams,
    },
};

// Password handling is done in master database, not tenant databases

/// Fetches user information based on query parameters.
///
/// This function queries the tenant database for user information using the provided query parameters.
/// If an `id` is specified in the query, it returns a single user.
/// If no `id` is specified, it checks for pagination parameters (`page` and `page_size`) to
/// determine whether to return a paginated list or all users.
///
/// # Arguments
///
/// * `params` - A `Query` extractor containing query parameters for user retrieval.
/// * `state` - The application state containing tenant manager.
/// * `tenant_context` - The tenant context extracted from JWT token.
///
/// # Returns
///
/// * `Result<impl IntoResponse>` - If successful, returns an HTTP response with a status code and
///   serialized JSON data of the user(s). Contains either a single user or multiple users
///   based on the query parameters. Returns an error response if any database operation fails.
#[instrument(skip(state))]
pub async fn users_index(
    Query(params): Query<UsersUrlParams>,
    Extension(state): Extension<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!(
        id = ?params.id,
        page = ?params.page,
        page_size = ?params.page_size,
        tenant_id = %tenant_context.tenant_id,
        "Fetching users"
    );

    // Get tenant database connection
    let tenant_db = state
        .tenant_manager
        .get_tenant_connection(&tenant_context.tenant_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to get tenant database connection");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database connection error".to_string(),
            )
        })?;

    // Check if id is present.
    match params.id {
        // If id is present, return a single User.
        Some(id) => {
            info!(user_id = id, "Fetching single user");

            let query = Entity::find_by_id(&id)
                .one(&tenant_db)
                .await;

            match query {
                Ok(Some(user)) => {
                    info!(
                        user_id = user.id,
                        email = %user.email,
                        "Successfully fetched user"
                    );

                    let user_response = UserResponse {
                        id: user.id,
                        email: user.email,
                        first_name: user.first_name,
                        last_name: user.last_name,
                        tenant_id: tenant_context.tenant_id.clone(),
                        created_at: user.created_at,
                        updated_at: user.updated_at,
                    };

                    Ok((
                        StatusCode::OK,
                        Json(UsersResponseType::SingleUser(user_response)),
                    ))
                }
                Ok(None) => {
                    error!(user_id = id, "User not found");
                    Err((
                        StatusCode::NOT_FOUND,
                        format!("User with ID {} not found", id),
                    ))
                }
                Err(e) => {
                    error!(user_id = id, error = %e, "Database error while fetching user");
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Database error".to_string(),
                    ))
                }
            }
        }
        // If id is not present proceed to return multiple Users.
        None => {
            info!("Fetching multiple users");

            // Check if pagination parameters are present.
            match params.page {
                // If pagination parameters are present, return a paginated list of Users.
                Some(page) => {
                    info!(page = page, page_size = ?params.page_size, "Fetching paginated users");

                    let mut query = Entity::find();

                    // Apply filters
                    if let Some(email) = params.email {
                        query = query.filter(Column::Email.contains(email));
                    }
                    if let Some(first_name) = params.first_name {
                        query = query.filter(Column::FirstName.contains(first_name));
                    }
                    if let Some(last_name) = params.last_name {
                        query = query.filter(Column::LastName.contains(last_name));
                    }

                    let paginator = query
                        .order_by_desc(Column::Id)
                        .paginate(&tenant_db, params.page_size.unwrap_or(25) as u64);
                    
                    let total_count = paginator.num_items().await.unwrap_or(0);
                    let users = paginator
                        .fetch_page((page - 1) as u64)
                        .await;

                    match users {
                        Ok(users_result) => {

                            let user_responses: Vec<UserResponse> = users_result
                                .into_iter()
                                .map(|user| UserResponse {
                                    id: user.id,
                                    email: user.email,
                                    first_name: user.first_name,
                                    last_name: user.last_name,
                                    tenant_id: tenant_context.tenant_id.clone(),
                                    created_at: user.created_at,
                                    updated_at: user.updated_at,
                                })
                                .collect();

                            info!(
                                page = page,
                                user_count = user_responses.len(),
                                total_count = total_count,
                                "Successfully fetched paginated users"
                            );

                            Ok((
                                StatusCode::OK,
                                Json(UsersResponseType::PaginatedUsers {
                                    users: user_responses,
                                    total_count,
                                    page,
                                    page_size: params.page_size.unwrap_or(25),
                                }),
                            ))
                        }
                        Err(e) => {
                            error!(page = page, error = %e, "Database error while fetching paginated users");
                            Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Database error".to_string(),
                            ))
                        }
                    }
                }
                // If pagination parameters are not present, return all Users.
                None => {
                    info!("Fetching all users");

                    let mut query = Entity::find();

                    // Apply filters
                    if let Some(email) = params.email {
                        query = query.filter(Column::Email.contains(email));
                    }
                    if let Some(first_name) = params.first_name {
                        query = query.filter(Column::FirstName.contains(first_name));
                    }
                    if let Some(last_name) = params.last_name {
                        query = query.filter(Column::LastName.contains(last_name));
                    }

                    let users = query
                        .order_by_desc(Column::Id)
                        .all(&tenant_db)
                        .await;

                    match users {
                        Ok(users_result) => {
                            let user_responses: Vec<UserResponse> = users_result
                                .into_iter()
                                .map(|user| UserResponse {
                                    id: user.id,
                                    email: user.email,
                                    first_name: user.first_name,
                                    last_name: user.last_name,
                                    tenant_id: tenant_context.tenant_id.clone(),
                                    created_at: user.created_at,
                                    updated_at: user.updated_at,
                                })
                                .collect();

                            info!(
                                user_count = user_responses.len(),
                                "Successfully fetched all users"
                            );
                            Ok((
                                StatusCode::OK,
                                Json(UsersResponseType::MultipleUsers(user_responses)),
                            ))
                        }
                        Err(e) => {
                            error!(error = %e, "Database error while fetching all users");
                            Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Database error".to_string(),
                            ))
                        }
                    }
                }
            }
        }
    }
}

/// Creates a new user with the given information.
///
/// This function takes a `UsersRequestBody` JSON object as input and creates a new user in the tenant database.
///
/// # Arguments
///
/// * `state` - The application state containing tenant manager.
/// * `tenant_context` - The tenant context extracted from JWT token.
/// * `input` - A `UsersRequestBody` JSON object containing the user information.
///
/// # Returns
///
/// * `Result<impl IntoResponse>` - If successful, returns an HTTP response with a status code of
///   `201 Created` and serialized JSON data of the created user.
#[instrument(skip(state))]
pub async fn users_create(
    Extension(state): Extension<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(input): Json<UsersRequestBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!("Creating new user");

    // Validate required fields
    let email = input.email.ok_or_else(|| {
        error!("Missing email in user creation request");
        (StatusCode::BAD_REQUEST, "Email is required".to_string())
    })?;

    // Note: Authentication and passwords are handled in master database.
    // This endpoint manages tenant-specific user profile data only.

    let first_name = input.first_name.ok_or_else(|| {
        error!("Missing first_name in user creation request");
        (
            StatusCode::BAD_REQUEST,
            "First name is required".to_string(),
        )
    })?;

    let last_name = input.last_name.ok_or_else(|| {
        error!("Missing last_name in user creation request");
        (StatusCode::BAD_REQUEST, "Last name is required".to_string())
    })?;

    info!(
        email = %email,
        first_name = %first_name,
        last_name = %last_name,
        tenant_id = %tenant_context.tenant_id,
        "Creating user with validated data"
    );

    // Get tenant database connection
    let tenant_db = state
        .tenant_manager
        .get_tenant_connection(&tenant_context.tenant_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to get tenant database connection");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database connection error".to_string(),
            )
        })?;

    // Note: Password handling should be done via master database auth endpoints.
    // This endpoint creates tenant-specific user profile data only.

    // Create user profile in tenant database
    let user = ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        email: Set(email.clone()),
        first_name: Set(first_name.clone()),
        last_name: Set(last_name.clone()),
        ..Default::default()
    };

    match user.insert(&tenant_db).await {
        Ok(created_user) => {
            info!(
                user_id = created_user.id,
                email = %created_user.email,
                "User created successfully"
            );

            let user_response = UserResponse {
                id: created_user.id,
                email: created_user.email,
                first_name: created_user.first_name,
                last_name: created_user.last_name,
                tenant_id: tenant_context.tenant_id.clone(),
                created_at: created_user.created_at,
                updated_at: created_user.updated_at,
            };

            Ok((StatusCode::CREATED, Json(user_response)))
        }
        Err(e) => {
            error!(
                error = %e,
                email = %email,
                "Failed to create user in database"
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ))
        }
    }
}

/// Updates a user by providing a JSON request body with the fields that should be updated.
///
/// The JSON request body should contain the `id` field of the user to be updated.
///
/// # Arguments
///
/// * `state` - The application state containing tenant manager.
/// * `tenant_context` - The tenant context extracted from JWT token.
/// * `updates` - A `UsersRequestBody` JSON object containing the user updates.
///
/// # Returns
///
/// * `Result<impl IntoResponse>` - If successful, returns an HTTP response with a status code of
///   `200 OK` and serialized JSON data of the updated user.
#[instrument(skip(state))]
pub async fn users_update(
    Extension(state): Extension<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(updates): Json<UsersRequestBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let None = updates.id {
        error!("Missing user ID in update request");
        return Err((StatusCode::BAD_REQUEST, "User ID is required".to_string()));
    }

    let user_id = updates.id.unwrap();
    info!(user_id = user_id, "Updating user");

    // Get tenant database connection
    let tenant_db = state
        .tenant_manager
        .get_tenant_connection(&tenant_context.tenant_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to get tenant database connection");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database connection error".to_string(),
            )
        })?;

    let original_user = match Entity::find_by_id(&user_id)
        .one(&tenant_db)
        .await
    {
        Ok(Some(user)) => {
            info!(user_id = user_id, "Found user for update");
            user
        }
        Ok(None) => {
            error!(user_id = user_id, "User not found for update");
            return Err((
                StatusCode::NOT_FOUND,
                "User with provided ID not found".to_string(),
            ));
        }
        Err(e) => {
            error!(user_id = user_id, error = %e, "Database error while finding user for update");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ));
        }
    };

    let mut user: ActiveModel = original_user.clone().into();

    if let Some(email) = updates.email {
        info!(user_id = user_id, email = %email, "Updating email");
        user.email = Set(email);
    }

    // Note: Password updates should be done via master database auth endpoints

    if let Some(first_name) = updates.first_name {
        info!(user_id = user_id, first_name = %first_name, "Updating first_name");
        user.first_name = Set(first_name);
    }

    if let Some(last_name) = updates.last_name {
        info!(user_id = user_id, last_name = %last_name, "Updating last_name");
        user.last_name = Set(last_name);
    }

    match user.update(&tenant_db).await {
        Ok(updated_user) => {
            info!(
                user_id = updated_user.id,
                email = %updated_user.email,
                "User updated successfully"
            );

            let user_response = UserResponse {
                id: updated_user.id,
                email: updated_user.email,
                first_name: updated_user.first_name,
                last_name: updated_user.last_name,
                tenant_id: tenant_context.tenant_id.clone(),
                created_at: updated_user.created_at,
                updated_at: updated_user.updated_at,
            };

            Ok((StatusCode::OK, Json(user_response)))
        }
        Err(e) => {
            error!(
                user_id = user_id,
                error = %e,
                "Failed to update user in database"
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ))
        }
    }
}

/// Deletes a user from the database.
///
/// This function takes a `UsersRequestBody` JSON object as input and deletes the corresponding
/// user from the tenant database.
///
/// # Arguments
///
/// * `state` - The application state containing tenant manager.
/// * `tenant_context` - The tenant context extracted from JWT token.
/// * `input` - A `UsersRequestBody` JSON object containing the user to be deleted.
///
/// # Returns
///
/// * `Result<impl IntoResponse>` - If successful, returns an HTTP response with a status code of
///   `200 OK` and a message indicating that the user was deleted successfully.
#[instrument(skip(state))]
pub async fn users_delete(
    Extension(state): Extension<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(input): Json<UsersRequestBody>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let None = input.id {
        error!("Missing user ID in delete request");
        return Err((StatusCode::BAD_REQUEST, "User ID is required".to_string()));
    }

    let user_id = input.id.unwrap();
    info!(user_id = user_id, "Deleting user");

    // Get tenant database connection
    let tenant_db = state
        .tenant_manager
        .get_tenant_connection(&tenant_context.tenant_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to get tenant database connection");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database connection error".to_string(),
            )
        })?;

    match Entity::delete_by_id(&user_id)
        .exec(&tenant_db)
        .await
    {
        Ok(_) => {
            info!(user_id = user_id, "User deleted successfully");
            Ok((StatusCode::OK, "User deleted successfully".to_string()))
        }
        Err(e) => {
            error!(user_id = user_id, error = %e, "Failed to delete user from database");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ))
        }
    }
}

/// Returns the count of users in the tenant database.
///
/// This function takes a `UsersCountUrlParams` object as input and returns the count of users
/// in the tenant database with optional filtering.
///
/// # Arguments
///
/// * `state` - The application state containing tenant manager.
/// * `tenant_context` - The tenant context extracted from JWT token.
/// * `params` - A `UsersCountUrlParams` object containing filter parameters.
///
/// # Returns
///
/// * `Result<impl IntoResponse>` - If successful, returns an HTTP response with a status code of
///   `200 OK` and a JSON response with the count of users.
#[instrument(skip(state))]
pub async fn users_count(
    Extension(state): Extension<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<UsersCountUrlParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!(
        tenant_id = %tenant_context.tenant_id,
        email = ?params.email,
        first_name = ?params.first_name,
        last_name = ?params.last_name,
        "Counting users"
    );

    // Get tenant database connection
    let tenant_db = state
        .tenant_manager
        .get_tenant_connection(&tenant_context.tenant_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to get tenant database connection");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database connection error".to_string(),
            )
        })?;

    let mut query = Entity::find();

    // Apply filters
    if let Some(email) = params.email {
        query = query.filter(Column::Email.contains(email));
    }
    if let Some(first_name) = params.first_name {
        query = query.filter(Column::FirstName.contains(first_name));
    }
    if let Some(last_name) = params.last_name {
        query = query.filter(Column::LastName.contains(last_name));
    }

    let count = query.count(&tenant_db).await;

    match count {
        Ok(count_result) => {
            info!(count = count_result, "Successfully counted users");
            Ok((StatusCode::OK, Json(count_result)))
        }
        Err(e) => {
            error!(error = %e, "Database error while counting users");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ))
        }
    }
}
