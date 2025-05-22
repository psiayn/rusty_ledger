#!/bin/bash
set -e

echo "Waiting for database to be ready..."
for i in {1..30}; do
  if sqlx database create && sqlx migrate run; then
    echo "Database migrations completed successfully!"
    break
  fi
  echo "Database not ready yet, retrying in 1 second..."
  sleep 1
  if [ $i -eq 30 ]; then
    echo "Database connection failed after 30 attempts."
    exit 1
  fi
done

echo "Starting Rusty Ledger application..."
exec /app/rusty_ledger 