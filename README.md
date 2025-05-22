# Rusty Ledger

A Rust-based backend banking application with a RESTful API for managing transactions, user accounts, and balances.

## Features

- User authentication with JWT
- Transaction management
- Account balance tracking
- Query functionality for transactions
- PostgreSQL database for persistence

## API Documentation

See [API.md](./API.md) for detailed documentation of all endpoints.

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

### Project Structure

- `src/api/` - API route handlers
  - `user.rs` - User registration, login, and profile management
  - `account.rs` - Account balance operations
  - `transaction.rs` - Transaction creation and querying
- `src/middleware/` - Application middleware (authentication)
- `migrations/` - Database migration files

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

See the [LICENSE](LICENSE) file for details. 
