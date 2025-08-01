# Multi-Tenant API with Rust, Axum, and Sea-ORM

A production-ready multi-tenant web application built with Rust, Axum, and Sea-ORM. Each tenant gets their own database for complete data isolation.

## Architecture

### Database Structure
```
PostgreSQL Cluster
├── postgres (default database)
├── myapp_database (master database - manages tenants)
├── tenant_company_a (tenant A's database)
├── tenant_company_b (tenant B's database)
└── tenant_startup_xyz (tenant C's database)
```

### Key Components

1. **Master Database**: Manages tenant information, user authentication, and permissions
2. **Tenant Databases**: Each tenant gets their own database with their data
3. **JWT Authentication**: Secure token-based authentication with tenant context
4. **Dynamic Connection Management**: Automatic tenant database connection handling

## Setup

### 1. Environment Configuration

Create a `.env` file with the following variables:

```env
# Database Configuration
MASTER_DATABASE_URL=postgresql://myapp:your_password@localhost:5432/myapp_database
DB_USERNAME=myapp
DB_PASSWORD=your_password
DB_HOST=localhost
DB_PORT=5432

# JWT Configuration
JWT_SECRET=your-super-secret-jwt-key-here-make-it-long-and-random-at-least-32-characters
JWT_EXPIRATION=3600

# CORS Configuration
CORS_ORIGINS=http://localhost:3000,http://localhost:3001
```

### 2. Database Setup

```bash
# Create database user
sudo -u postgres createuser --interactive myapp

# Create master database
sudo -u postgres createdb myapp_database

# Set password
sudo -u postgres psql -c "ALTER USER myapp WITH PASSWORD 'your_password';"

# Grant permissions
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE myapp_database TO myapp;"
```

### 3. Run the Application

```bash
cargo run
```

The server will start on `http://localhost:8000`

## API Endpoints

### Public Routes (No Authentication Required)

- `GET /` - Health check
- `POST /auth/login` - User login
- `POST /auth/register` - User registration
- `POST /tenants` - Create new tenant

### Protected Routes (Require JWT Authentication)

- `GET /api/users` - Get all users (tenant-specific)
- `POST /api/users` - Create new user (tenant-specific)
- `GET /api/users/:id` - Get specific user (tenant-specific)
- `PUT /api/users/:id` - Update user (tenant-specific)
- `DELETE /api/users/:id` - Delete user (tenant-specific)

## How It Works

### 1. Tenant Creation

When a new tenant is created:

1. **Master Database**: Tenant record is created in the master database
2. **Tenant Database**: A new database is created for the tenant
3. **Migrations**: Tenant-specific migrations are run on the new database

### 2. Authentication Flow

1. **Login**: User provides email/password
2. **Validation**: Credentials are checked against master database
3. **JWT Token**: Token is created with tenant context and permissions
4. **Requests**: All subsequent requests include the JWT token

### 3. Request Processing

1. **Token Extraction**: JWT token is extracted from Authorization header
2. **Token Validation**: Token is validated and decoded
3. **Tenant Resolution**: Tenant ID is extracted from token
4. **Database Connection**: Tenant-specific database connection is established
5. **Permission Check**: User permissions are verified
6. **Request Processing**: Request is processed with tenant context

## Example Usage

### Create a Tenant

```bash
curl -X POST http://localhost:8000/tenants \
  -H "Content-Type: application/json" \
  -d '{
    "id": "acme_corp",
    "name": "Acme Corporation"
  }'
```

### Register a User

```bash
curl -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john@acme.com",
    "password": "password123",
    "first_name": "John",
    "last_name": "Doe"
  }'
```

### Login

```bash
curl -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john@acme.com",
    "password": "password123"
  }'
```

### Access Protected Endpoint

```bash
curl -X GET http://localhost:8000/api/users \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## Security Features

- **JWT Authentication**: Secure token-based authentication
- **Tenant Isolation**: Complete database separation per tenant
- **Permission System**: Fine-grained access control
- **CORS Protection**: Configurable cross-origin resource sharing
- **Input Validation**: Request validation and sanitization

## Production Considerations

1. **Password Hashing**: Implement proper password hashing (e.g., Argon2)
2. **Rate Limiting**: Add rate limiting to prevent abuse
3. **Logging**: Implement comprehensive logging
4. **Monitoring**: Add health checks and monitoring
5. **Backup Strategy**: Implement database backup strategy
6. **SSL/TLS**: Use HTTPS in production
7. **Connection Pooling**: Optimize database connection pooling

## Development

### Project Structure

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library exports
├── config.rs            # Configuration management
├── auth.rs              # JWT authentication
├── tenant_manager.rs    # Dynamic database connection management
├── master.rs            # Master database operations
├── tenant.rs            # Tenant database operations
└── shared.rs            # Shared types and utilities

master_migration/        # Master database migrations
tenant_migration/        # Tenant database migrations
```

### Adding New Features

1. **Master Database**: Add tables to `master_migration/`
2. **Tenant Database**: Add tables to `tenant_migration/`
3. **API Endpoints**: Add routes in `main.rs`
4. **Business Logic**: Add services in `master.rs` or `tenant.rs`

## Troubleshooting

### Common Issues

1. **Database Connection Errors**: Check PostgreSQL is running and credentials are correct
2. **Migration Errors**: Ensure master database exists and user has proper permissions
3. **JWT Errors**: Verify JWT_SECRET is set and token format is correct
4. **Permission Errors**: Check user permissions in master database

### Debug Mode

Set `RUST_LOG=debug` to enable detailed logging:

```bash
RUST_LOG=debug cargo run
```

## License

MIT License 