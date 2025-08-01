# Multi-Tenant Rust Application

A production-ready, high-performance multi-tenant web application built with Rust, featuring complete database isolation per tenant, JWT authentication, and automatic timestamp management.

## 🚀 Features

- **Database-per-Tenant Architecture**: Complete data isolation with dedicated PostgreSQL database for each tenant
- **JWT Authentication**: Secure token-based authentication with tenant context embedding
- **Automatic Timestamp Management**: UTC timestamps with `ActiveModelBehavior` for all entities
- **High Performance**: Built with Rust's zero-cost abstractions and async/await
- **Type Safety**: Compile-time SQL injection prevention through Sea-ORM
- **Production Ready**: Argon2 password hashing, CORS protection, and comprehensive error handling

## 🛠️ Technology Stack

- **Language**: Rust 2021 Edition
- **Web Framework**: Axum 0.7.4
- **ORM**: Sea-ORM 0.12.14
- **Database**: PostgreSQL 15+
- **Authentication**: JWT (jsonwebtoken 9.2.0)
- **Password Hashing**: Argon2 (argon2 0.5.3)
- **Async Runtime**: Tokio 1.36.0

## 📋 Table of Contents

- [Architecture](#architecture)
- [Prerequisites](#prerequisites)
- [Setup](#setup)
- [API Documentation](#api-documentation)
- [Development](#development)
- [Security](#security)
- [Testing](#testing)
- [Production Deployment](#production-deployment)
- [Known Issues](#known-issues)
- [Troubleshooting](#troubleshooting)

## 🏗️ Architecture

### Database Architecture

```
PostgreSQL Cluster
│
├── postgres (system database)
│
├── rust_multi_tenant (master database)
│   ├── tenants (table)
│   │   ├── id (string, PK)
│   │   ├── name (string)
│   │   ├── status (string)
│   │   ├── created_at (timestamp)
│   │   └── updated_at (timestamp)
│   │
│   ├── users (table)
│   │   ├── id (string, PK)
│   │   ├── tenant_id (string, FK)
│   │   ├── email (string, unique)
│   │   ├── password_hash (string)
│   │   ├── permissions (json)
│   │   ├── created_at (timestamp)
│   │   └── updated_at (timestamp)
│   │
│   └── permissions (table)
│       ├── id (string, PK)
│       ├── name (string, unique)
│       ├── description (string)
│       └── created_at (timestamp)
│
├── tenant_company_a (tenant database)
│   ├── users (table - profile data)
│   ├── products (table)
│   └── orders (table)
│
└── tenant_company_b (tenant database)
    ├── users (table - profile data)
    ├── products (table)
    └── orders (table)
```

### Authentication Flow

```
┌──────────┐     ┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│  Client  │────▶│ Auth Router │────▶│ Master Service│────▶│ Master DB   │
└──────────┘     └─────────────┘     └──────────────┘     └─────────────┘
     │                                        │
     │ 1. POST /auth/login                   │ 3. Verify credentials
     │    {email, password}                  │    Generate JWT with tenant_id
     │                                        │
     │◀───────────────────────────────────────│
     │ 2. JWT Token                          │
     │    {user_id, tenant_id, permissions}  │
     ▼                                        ▼
```

### Request Processing Flow

```
┌──────────┐     ┌────────────┐     ┌──────────────┐     ┌─────────────┐
│  Client  │────▶│ Middleware │────▶│  Controller  │────▶│ Tenant DB   │
└──────────┘     └────────────┘     └──────────────┘     └─────────────┘
     │                  │                     │
     │ 1. Request +     │ 2. Extract JWT     │ 4. Execute query on
     │    JWT Token     │    Validate token  │    tenant-specific DB
     │                  │    Inject context  │
     │◀─────────────────┼─────────────────────│
     │ 5. Response      │ 3. Route to handler│
     ▼                  ▼                     ▼
```

### Project Structure

```
rust_multi_tenant/
├── Cargo.toml                      # Workspace configuration
├── src/
│   ├── main.rs                     # Application entry point
│   ├── lib.rs                      # Library exports
│   ├── entities/
│   │   ├── master/                 # Master database entities
│   │   │   ├── tenants.rs         # Generated by sea-orm-cli
│   │   │   ├── users.rs           # With ActiveModelBehavior
│   │   │   └── permissions.rs
│   │   └── tenant/                 # Tenant database entities
│   │       ├── users.rs           # Profile data only
│   │       ├── products.rs
│   │       └── orders.rs
│   ├── controllers/
│   │   ├── auth/                   # Authentication endpoints
│   │   ├── tenants/               # Tenant management
│   │   └── users/                 # User CRUD operations
│   ├── middlewares/
│   │   ├── auth.rs                # JWT validation & context
│   │   └── cors.rs                # CORS configuration
│   ├── multi_tenancy/
│   │   ├── tenant_manager.rs      # Connection pool management
│   │   ├── master.rs              # Master DB operations
│   │   └── tenant.rs              # Tenant DB operations
│   └── types/                      # Shared types and DTOs
├── master_migration/               # Master database migrations
│   └── src/
│       ├── m20240101_000001_create_tenants_table.rs
│       ├── m20240101_000002_create_users_table.rs
│       └── m20240101_000003_create_permissions_table.rs
└── tenant_migration/               # Tenant database migrations
    └── src/
        ├── m20240101_000001_create_users_table.rs
        ├── m20240101_000002_create_products_table.rs
        └── m20240101_000003_create_orders_table.rs
```

## 📚 Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- PostgreSQL 15+ (with superuser access for database creation)
- Sea-ORM CLI: `cargo install sea-orm-cli`

## 🚀 Setup

### 1. Clone the Repository

```bash
git clone <repository-url>
cd rust_multi_tenant
```

### 2. Environment Configuration

Create a `.env` file in the project root:

```env
# Database Configuration
MASTER_DATABASE_URL=postgresql://pgroot:admin@localhost:5432/rust_multi_tenant
DB_USERNAME=pgroot
DB_PASSWORD=admin
DB_HOST=localhost
DB_PORT=5432

# JWT Configuration
JWT_SECRET=your-super-secret-jwt-key-here-make-it-long-and-random-at-least-32-characters
JWT_EXPIRATION=3600

# CORS Configuration
CORS_ORIGINS=http://localhost:3000,http://localhost:3001

# Logging
RUST_LOG=debug
```

### 3. Database Setup

```bash
# Create the master database
createdb rust_multi_tenant

# Run master database migrations
sea-orm-cli migrate up -d master_migration

# Generate entities from the database (optional - already included)
sea-orm-cli generate entity -u $MASTER_DATABASE_URL -o src/entities/master --lib
```

### 4. Run the Application

```bash
cargo run
```

The server will start on `http://localhost:3000`

## 📖 API Documentation

### Authentication Endpoints

#### Create Tenant
Creates a new tenant with a dedicated database.

```http
POST /tenants
Content-Type: application/json

{
  "id": "acme_corp",
  "name": "ACME Corporation"
}
```

**Response:**
```json
{
  "id": "acme_corp",
  "name": "ACME Corporation",
  "status": "active",
  "created_at": "2024-01-01T12:00:00",
  "updated_at": "2024-01-01T12:00:00"
}
```

#### Register User
**Note**: Currently hardcoded to use `demo_tenant`. See [Known Issues](#known-issues).

```http
POST /auth/register
Content-Type: application/json

{
  "email": "john@example.com",
  "password": "SecurePassword123!",
  "first_name": "John",
  "last_name": "Doe"
}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "john@example.com",
  "first_name": "John",
  "last_name": "Doe",
  "created_at": "2024-01-01T12:00:00",
  "updated_at": "2024-01-01T12:00:00"
}
```

#### Login
Authenticates a user and returns a JWT token.

```http
POST /auth/login
Content-Type: application/json

{
  "email": "john@example.com",
  "password": "SecurePassword123!"
}
```

**Response:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "john@example.com",
    "first_name": "John",
    "last_name": "Doe",
    "created_at": "2024-01-01T12:00:00",
    "updated_at": "2024-01-01T12:00:00"
  }
}
```

### Protected Endpoints (Require JWT)

All protected endpoints require the JWT token in the Authorization header:
```http
Authorization: Bearer <your-jwt-token>
```

#### List Users
Get all users in the tenant (profile data only).

```http
GET /api/users?page=1&page_size=25
```

**Response:**
```json
{
  "users": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "john@example.com",
      "first_name": "John",
      "last_name": "Doe",
      "tenant_id": "acme_corp",
      "created_at": "2024-01-01T12:00:00",
      "updated_at": "2024-01-01T12:00:00"
    }
  ],
  "total_count": 1,
  "page": 1,
  "page_size": 25
}
```

#### Get User Count
```http
GET /api/users/count?email=john
```

**Response:**
```json
{
  "count": 1
}
```

#### Create User Profile
Creates a user profile in the tenant database.

```http
POST /api/users
Content-Type: application/json

{
  "email": "jane@example.com",
  "first_name": "Jane",
  "last_name": "Smith"
}
```

#### Update User
```http
PATCH /api/users
Content-Type: application/json

{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "first_name": "John Updated"
}
```

#### Delete User
```http
DELETE /api/users?id=550e8400-e29b-41d4-a716-446655440000
```

### Error Responses

All endpoints return consistent error responses:

```json
{
  "error": "Unauthorized",
  "message": "Invalid or expired token",
  "status_code": 401
}
```

Common HTTP status codes:
- `400` - Bad Request (invalid input)
- `401` - Unauthorized (missing or invalid JWT)
- `403` - Forbidden (insufficient permissions)
- `404` - Not Found
- `500` - Internal Server Error

## 💻 Development

### Running Migrations

```bash
# Master database migrations
sea-orm-cli migrate up -d master_migration

# For tenant databases (run after tenant creation)
sea-orm-cli migrate up -d tenant_migration -u postgresql://pgroot:admin@localhost:5432/tenant_<tenant_id>
```

### Generating Entities

After modifying migrations:

```bash
# Regenerate master entities
sea-orm-cli generate entity -u $MASTER_DATABASE_URL -o src/entities/master --lib

# For tenant entities (example)
sea-orm-cli generate entity -u postgresql://pgroot:admin@localhost:5432/tenant_example -o src/entities/tenant --lib
```

### Code Organization Principles

1. **Controllers**: Handle HTTP requests and responses
2. **Services**: Business logic (master.rs, tenant.rs)
3. **Middlewares**: Cross-cutting concerns (auth, CORS)
4. **Entities**: Database models with `ActiveModelBehavior`
5. **Types**: DTOs and shared types

### Adding New Features

1. **For Master Database Features**:
   - Add migration in `master_migration/src/`
   - Regenerate entities
   - Add service methods in `src/multi_tenancy/master.rs`
   - Create controller in `src/controllers/`

2. **For Tenant Features**:
   - Add migration in `tenant_migration/src/`
   - Update tenant entities
   - Add service methods in `src/multi_tenancy/tenant.rs`
   - Ensure tenant context is used in controllers

## 🔒 Security

### Implemented Security Features

1. **Password Security**:
   - Argon2 hashing with salt
   - Never store plain text passwords
   - Password complexity requirements (implement in frontend)

2. **JWT Security**:
   - Short expiration times (1 hour default)
   - Tenant context embedded in token
   - Signature verification on every request

3. **Database Isolation**:
   - Complete separation between tenants
   - Connection validation before each query
   - Tenant status verification

4. **Input Validation**:
   - Type-safe request parsing
   - SQL injection prevention via Sea-ORM
   - CORS protection

### Security Best Practices

```rust
// Example: Secure password hashing
let salt = SaltString::generate(&mut OsRng);
let argon2 = Argon2::default();
let password_hash = argon2
    .hash_password(password.as_bytes(), &salt)?
    .to_string();

// Example: JWT validation in middleware
let token_data = decode::<Claims>(
    &token,
    &DecodingKey::from_secret(jwt_secret.as_ref()),
    &Validation::default()
)?;
```

### Additional Security Recommendations

1. **Rate Limiting**: Implement with `tower-governor`
2. **HTTPS**: Always use TLS in production
3. **Secrets Management**: Use environment variables or vault
4. **Audit Logging**: Log all authentication attempts
5. **Regular Updates**: Keep dependencies updated

## 🧪 Testing

### Testing Strategy

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_tenant_creation
```

### Example Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase};

    #[tokio::test]
    async fn test_tenant_creation() {
        // Setup mock database
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![/* mock data */]])
            .into_connection();

        // Test tenant creation
        let service = MasterService::new(db);
        let result = service.create_tenant(/* ... */).await;
        
        assert!(result.is_ok());
    }
}
```

### Integration Testing

Create `tests/integration_test.rs`:

```rust
use testcontainers::{clients, images::postgres::Postgres};

#[tokio::test]
async fn test_full_tenant_lifecycle() {
    let docker = clients::Cli::default();
    let postgres = docker.run(Postgres::default());
    
    // Test full lifecycle
    // 1. Create tenant
    // 2. Create user
    // 3. Login
    // 4. Access tenant resources
}
```

## 🚀 Production Deployment

### Docker Deployment

Create `Dockerfile`:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates
COPY --from=builder /app/target/release/rust_multi_tenant /usr/local/bin/
CMD ["rust_multi_tenant"]
```

Create `docker-compose.yml`:

```yaml
version: '3.8'
services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      - MASTER_DATABASE_URL=postgresql://postgres:password@db:5432/rust_multi_tenant
    depends_on:
      - db
  
  db:
    image: postgres:15
    environment:
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

### Performance Tuning

1. **Connection Pooling**:
   ```rust
   // In tenant_manager.rs
   const MAX_CONNECTIONS_PER_TENANT: u32 = 20; // Increase for high traffic
   const CONNECTION_TIMEOUT: u64 = 30; // Seconds
   ```

2. **Database Optimization**:
   - Add indexes on frequently queried columns
   - Use read replicas for tenant databases
   - Implement query result caching

3. **Monitoring**:
   ```toml
   # Add to Cargo.toml
   tracing-subscriber = "0.3"
   metrics = "0.21"
   ```

### Production Checklist

- [ ] Set strong JWT_SECRET (min 32 characters)
- [ ] Configure proper CORS origins
- [ ] Set up SSL/TLS certificates
- [ ] Implement rate limiting
- [ ] Configure log aggregation
- [ ] Set up database backups
- [ ] Implement health checks
- [ ] Configure monitoring/alerting
- [ ] Set up CI/CD pipeline
- [ ] Document runbooks

## ⚠️ Known Issues

### 1. Demo Tenant Hardcoding
Currently, user registration is hardcoded to use `demo_tenant`:

```rust
// In auth_controller.rs
let tenant_id = "demo_tenant"; // TODO: Implement tenant selection
```

**Workaround**: 
- Create a `demo_tenant` via `/tenants` endpoint before registering users
- Or modify the registration endpoint to accept `tenant_id`

**Planned Fix**:
- Add tenant selection during registration
- Implement tenant invitation system
- Support multi-tenant user accounts

### 2. Connection Pool Management
The current implementation clears all connections when limit is reached:

```rust
if self.connections.lock().await.len() >= MAX_CONNECTIONS {
    connections.clear(); // Naive approach
}
```

**Planned Fix**: Implement proper LRU cache with the `lru` crate.

## 🔧 Troubleshooting

### Common Issues

1. **Database Connection Failed**
   ```
   Error: Connection refused (os error 111)
   ```
   - Ensure PostgreSQL is running: `sudo systemctl status postgresql`
   - Check connection parameters in `.env`

2. **Migration Failed**
   ```
   Error: permission denied to create database
   ```
   - Ensure user has CREATEDB permission:
     ```sql
     ALTER USER pgroot CREATEDB;
     ```

3. **JWT Token Invalid**
   ```
   Error: InvalidToken
   ```
   - Check JWT_SECRET matches between restarts
   - Verify token hasn't expired

4. **Tenant Database Not Found**
   ```
   Error: database "tenant_xyz" does not exist
   ```
   - Ensure tenant was created via `/tenants` endpoint
   - Check tenant status is "active" in master database

### Debug Mode

Enable detailed logging:

```bash
RUST_LOG=debug,sea_orm=debug cargo run
```

### Database Inspection

```sql
-- Check tenants
psql -U pgroot -d rust_multi_tenant -c "SELECT * FROM tenants;"

-- Check users in master
psql -U pgroot -d rust_multi_tenant -c "SELECT id, email, tenant_id FROM users;"

-- Check tenant database
psql -U pgroot -d tenant_acme_corp -c "\dt"
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Add tests for new features
- Update documentation

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Axum](https://github.com/tokio-rs/axum) - Ergonomic web framework
- [Sea-ORM](https://github.com/SeaQL/sea-orm) - Async ORM for Rust
- [Tokio](https://github.com/tokio-rs/tokio) - Async runtime
- Rust community for excellent documentation and support