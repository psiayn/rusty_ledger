# Rusty Ledger Code Documentation

This document provides an overview of the Rusty Ledger codebase structure and architecture.

## Project Architecture

Rusty Ledger is built using the following key technologies:

- **Axum**: Web framework for handling HTTP requests and routing
- **SQLx**: SQL toolkit for Rust, used for database interactions
- **PostgreSQL**: Database for storing user, account, and transaction data
- **JWT**: JSON Web Tokens for authentication
- **Tokio**: Asynchronous runtime for Rust

The application follows a modular architecture:

```
src/
├── api/              # API handlers for different routes
│   ├── user.rs       # User management (register, login, profile)
│   ├── account.rs    # Account operations (balance check/update)
│   ├── transaction.rs # Transaction operations (create, query)
│   └── mod.rs        # Module exports
├── middleware/       # Middleware components
│   └── auth.rs       # Authentication middleware
├── lib.rs            # Application routing and setup
├── main.rs           # Application entry point
└── state.rs          # Application state management
```

## Core Components

### Application State

The application state is defined in `state.rs` and contains a database connection pool that's shared across all routes:

```rust
pub struct AppState {
    pub db: sqlx::PgPool
}
```

### Main Entry Point

The `main.rs` file initializes the application:
1. Loads environment variables
2. Establishes a database connection
3. Creates the application state
4. Starts the HTTP server

### Routing

Routes are defined in `lib.rs`, which sets up the API endpoints and connects them to their respective handlers. The routes follow a RESTful pattern under the `/api/v1/` prefix.

### API Handlers

API handlers are organized by domain:

#### User Management (`api/user.rs`)
- `register`: Handles user registration
- `login`: Authenticates users and issues JWT tokens
- `update_profile`: Updates user profile information

#### Account Management (`api/account.rs`)
- `check_balance`: Retrieves account balance
- `update_balance`: Updates account balance

#### Transaction Management (`api/transaction.rs`)
- `create`: Creates new transactions
- `get_all`: Retrieves all transactions for a user
- `query`: Queries transactions based on filters

### Authentication

Authentication is implemented as middleware in `middleware/auth.rs`. It:
1. Extracts the JWT token from the Authorization header
2. Validates the token
3. Extracts the user ID and adds it to the request extension
4. Allows or denies access to protected routes

## Database Schema

The database schema is managed through migrations in the `migrations/` directory:

- Users table: Stores user information (username, password hash, email)
- Accounts table: Stores account information (balance, owner)
- Transactions table: Stores transaction records (amount, description, timestamp)

## Error Handling

The application uses the `anyhow` crate for error handling, providing detailed error context throughout the application.

## Testing

Tests are located in the `tests/` directory and are organized to mirror the main code structure.

## Building and Deployment

The application can be built and deployed using:
- Cargo for local development
- Docker and Docker Compose for containerized deployment

See the README.md for detailed instructions on building and running the application. 