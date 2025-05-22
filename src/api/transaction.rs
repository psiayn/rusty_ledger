use axum::{extract::{Query, State}, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::{types::{BigDecimal, Uuid}, Pool, Postgres, Error as SqlxError};
use time::OffsetDateTime;

use crate::state;

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateTransReq {
    from_account_id: Uuid,
    to_account_id: Uuid,
    amount: BigDecimal,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GetTransReq {
    account_id: Uuid
}

#[derive(Clone, Serialize, Deserialize)]
struct BalanceResult {
    account_id: Uuid,
    balance: BigDecimal
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Transaction {
    id: Uuid,
    from_account_id: Uuid,
    to_account_id: Uuid,
    amount: BigDecimal,
    created_at: Option<OffsetDateTime>
}

async fn update_balance(trans: CreateTransReq, pool: Pool<Postgres>) -> anyhow::Result<()> {
    let from_balance = sqlx::query_as!(
        BalanceResult,
        "SELECT account_id, balance FROM account_balances where account_id = $1",
        trans.from_account_id
    ).fetch_one(&pool).await
     .map_err(|e| anyhow::anyhow!("Failed to fetch source account balance: {}", e))?;
    
    let to_balance = sqlx::query_as!(
        BalanceResult,
        "SELECT account_id, balance FROM account_balances where account_id = $1",
        trans.to_account_id
    ).fetch_one(&pool).await
     .map_err(|e| anyhow::anyhow!("Failed to fetch destination account balance: {}", e))?;

    let new_from_acc_balance = from_balance.balance - trans.clone().amount;
    let new_to_acc_balance = to_balance.balance + trans.clone().amount;

    if (new_from_acc_balance > BigDecimal::from(0)) && (new_to_acc_balance > BigDecimal::from(0)) {
        let new_from_balance = sqlx::query_scalar!(
            "UPDATE account_balances SET balance = $1 where account_id = $2 returning balance",
            new_from_acc_balance,
            from_balance.account_id
        ).fetch_one(&pool).await
         .map_err(|e| anyhow::anyhow!("Failed to update source account balance: {}", e))?;
        
        let new_to_balance = sqlx::query_scalar!(
            "UPDATE account_balances SET balance = $1 where account_id = $2 returning balance",
            new_to_acc_balance,
            to_balance.account_id
        ).fetch_one(&pool).await
         .map_err(|e| anyhow::anyhow!("Failed to update destination account balance: {}", e))?;
        
        println!("from_balance = {new_from_balance}, to_balance = {new_to_balance}, amount = {}", trans.amount.to_string());
        Ok(())
    } else {
        Err(anyhow::anyhow!("Insufficient balance for transaction. From account balance would be {new_from_acc_balance}, to account balance would be {new_to_acc_balance}"))
    }
}

#[axum::debug_handler]
pub async fn create(State(state): State<state::AppState>, Json(req): Json<CreateTransReq>) -> Result<Json<String>, (StatusCode, String)> {
    let pool = state.db;

    let transaction_id = sqlx::query_scalar!(
        "INSERT INTO transactions (from_account_id, to_account_id, amount ) values ($1, $2, $3) returning id",
        req.from_account_id,
        req.to_account_id,
        req.amount,
    ).fetch_one(&pool).await
     .map_err(|e| {
        let error_msg = format!("Failed to create transaction: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, error_msg)
     })?;

    match update_balance(req, pool).await {
        Ok(_) => Ok(Json(format!("Transaction created successfully with ID: {}", transaction_id))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string()))
    }
}

pub async fn get_all(State(state): State<state::AppState>) -> Result<Json<Vec<Transaction>>, (StatusCode, String)> {
    let pool = state.db;

    let res = sqlx::query_as!(
        Transaction,
        "SELECT * FROM transactions;"
    ).fetch_all(&pool)
     .await
     .map_err(|e| {
        let error_msg = format!("Failed to fetch transactions: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, error_msg)
     })?;

    Ok(Json(res))
}

pub async fn query(State(state): State<state::AppState>, Query(req): Query<GetTransReq>) -> Result<Json<Vec<Transaction>>, (StatusCode, String)> {
    let pool = state.db;

    let res = sqlx::query_as!(
        Transaction,
        "SELECT * FROM transactions where from_account_id = $1 OR to_account_id = $1",
        req.account_id
    ).fetch_all(&pool).await
     .map_err(|e| {
        match e {
            SqlxError::RowNotFound => (
                StatusCode::NOT_FOUND, 
                format!("No transactions found for account ID: {}", req.account_id)
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch transactions: {}", e)
            )
        }
     })?;

    if res.is_empty() {
        return Err((StatusCode::NOT_FOUND, format!("No transactions found for account ID: {}", req.account_id)));
    }

    Ok(Json(res))
}
