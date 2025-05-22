# Rusty Ledger API Documentation

This document provides detailed information about the Rusty Ledger API endpoints.

## Base URL

All endpoints are relative to: `http://localhost:3000/api/v1`

## Authentication

Most endpoints require authentication using JWT (JSON Web Token). Include the token in the `Authorization` header:

```
Authorization: Bearer <your_token>
```

You can obtain a token by calling the login endpoint.

## Endpoints

### User Management

#### Register User
- **URL**: `/user/register`
- **Method**: `POST`
- **Authentication**: Not required
- **Request Body**:
  ```json
  {
    "username": "string",
    "password": "string",
    "email": "string"
  }
  ```
- **Response**:
  ```json
  {
    "message": "User registered successfully",
    "user_id": "uuid"
  }
  ```

#### Login
- **URL**: `/user/login`
- **Method**: `POST`
- **Authentication**: Not required
- **Request Body**:
  ```json
  {
    "username": "string",
    "password": "string"
  }
  ```
- **Response**:
  ```json
  {
    "token": "jwt_token_string",
    "user_id": "uuid"
  }
  ```

#### Update Profile
- **URL**: `/user/updateProfile`
- **Method**: `GET`
- **Authentication**: Required
- **Query Parameters**:
  - `email`: (optional) New email address
  - `username`: (optional) New username
- **Response**:
  ```json
  {
    "message": "Profile updated successfully"
  }
  ```

### Transaction Management

#### Create Transaction
- **URL**: `/transaction/create`
- **Method**: `POST`
- **Authentication**: Required
- **Request Body**:
  ```json
  {
    "amount": "decimal",
    "description": "string",
    "account_id": "uuid"
  }
  ```
- **Response**:
  ```json
  {
    "transaction_id": "uuid",
    "status": "success"
  }
  ```

#### Get All Transactions
- **URL**: `/transaction/all`
- **Method**: `GET`
- **Authentication**: Required
- **Response**:
  ```json
  [
    {
      "id": "uuid",
      "amount": "decimal",
      "description": "string",
      "created_at": "timestamp",
      "account_id": "uuid"
    }
  ]
  ```

#### Query Transactions
- **URL**: `/transaction/query`
- **Method**: `GET`
- **Authentication**: Required
- **Query Parameters**:
  - `start_date`: (optional) Filter by start date (ISO format)
  - `end_date`: (optional) Filter by end date (ISO format)
  - `min_amount`: (optional) Filter by minimum amount
  - `max_amount`: (optional) Filter by maximum amount
  - `description`: (optional) Filter by description (partial match)
- **Response**:
  ```json
  [
    {
      "id": "uuid",
      "amount": "decimal",
      "description": "string",
      "created_at": "timestamp",
      "account_id": "uuid"
    }
  ]
  ```

### Account Management

#### Check Balance
- **URL**: `/account/checkBalance`
- **Method**: `GET`
- **Authentication**: Required
- **Query Parameters**:
  - `account_id`: (optional) The account ID to check. If not provided, returns balance for all accounts.
- **Response**:
  ```json
  {
    "account_id": "uuid",
    "balance": "decimal",
    "last_updated": "timestamp"
  }
  ```
  or
  ```json
  [
    {
      "account_id": "uuid",
      "balance": "decimal",
      "last_updated": "timestamp"
    }
  ]
  ```

#### Update Balance
- **URL**: `/account/updateBalance`
- **Method**: `POST`
- **Authentication**: Required
- **Request Body**:
  ```json
  {
    "account_id": "uuid",
    "amount": "decimal"
  }
  ```
- **Response**:
  ```json
  {
    "account_id": "uuid",
    "new_balance": "decimal",
    "status": "success"
  }
  ```

## Error Responses

All endpoints may return the following error responses:

### 400 Bad Request
```json
{
  "error": "Description of the error"
}
```

### 401 Unauthorized
```json
{
  "error": "Authentication failed"
}
```

### 404 Not Found
```json
{
  "error": "Resource not found"
}
```

### 500 Internal Server Error
```json
{
  "error": "Internal server error"
}
``` 