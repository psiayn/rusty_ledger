use std::fmt;

use axum::{extract::{Query, State}, Json, http::StatusCode};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state;

#[derive(Clone, Serialize, Deserialize)]
pub enum Types {
    Savings,
    Current,
    Salary,
    FD, // Fixed Deposit
    RD, // Recurring Deposit
}


impl fmt::Display for Types {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result  {
        match self {
            Types::Savings => write!(f, "savings"),
            Types::Current => write!(f, "current"),
            Types::Salary => write!(f, "salary"),
            Types::FD => write!(f, "fd"),
            Types::RD => write!(f, "rd"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AccountBalanceReq {
    account_id: Uuid,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    account_id: Uuid,
    balance: BigDecimal,
}

pub async fn check_balance(
    State(state): State<state::AppState>, 
    Query(req): Query<AccountBalanceReq>
) -> Result<Json<AccountBalance>, (StatusCode, String)> {
    let pool = state.db;

    let user = sqlx::query_as!(
        AccountBalance,
        "SELECT account_id, balance FROM account_balances where account_id = $1",
        req.account_id
    ).fetch_one(&pool).await
     .map_err(|e| {
        match e {
            sqlx::Error::RowNotFound => (
                StatusCode::NOT_FOUND, 
                format!("Account with ID {} not found", req.account_id)
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR, 
                format!("Failed to fetch account balance: {}", e)
            )
        }
     })?;
    
    Ok(Json(user))
}

pub async fn update_balance(
    State(state): State<state::AppState>, 
    Json(req): Json<AccountBalance>
) -> Result<Json<AccountBalance>, (StatusCode, String)> {
    let pool = state.db;
    
    // First check if the account exists
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM account_balances WHERE account_id = $1)",
        req.account_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    if !exists.unwrap_or(false) {
        return Err((StatusCode::NOT_FOUND, format!("Account with ID {} not found", req.account_id)));
    }
    
    // Don't allow negative balances
    if req.balance < BigDecimal::from(0) {
        return Err((StatusCode::BAD_REQUEST, "Account balance cannot be negative".to_string()));
    }
    
    let balance = sqlx::query_scalar!(
        "UPDATE account_balances SET balance = $1 where account_id = $2 returning balance",
        req.balance,
        req.account_id
    ).fetch_one(&pool).await
     .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update account balance: {}", e)))?;
    
    let res = AccountBalance {
        account_id: req.account_id,
        balance: balance
    };

    Ok(Json(res))
}
