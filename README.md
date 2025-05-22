# Rusty Ledger

A Rust-based financial ledger application.

## Running the Application

### Using Docker Compose

To start the application with PostgreSQL database:

```bash
# Build and start the services
docker compose up -d

# View logs
docker compose logs -f
```

The application will:
1. Start the PostgreSQL database
2. Run database migrations automatically on startup
3. Start the application on port 3000

Access the application at http://localhost:3000

### Environment Variables

- `JWT_SECRET_KEY`: Secret key for JWT token generation (defaults to a development value if not provided)
- `DATABASE_URL`: PostgreSQL connection string (handled automatically in Docker Compose)

## Development

### Building Locally

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Run database migrations
sqlx database create
sqlx migrate run

# Run the application
cargo run
``` 
