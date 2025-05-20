use axum::{extract::{Query, State}, Json};
use serde::{Deserialize, Serialize};
use sqlx::{types::{BigDecimal, Uuid}, Pool, Postgres};
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

async fn update_balance(trans: CreateTransReq, pool: Pool<Postgres>) {
    let from_balance = sqlx::query_as!(
        BalanceResult,
        "SELECT account_id, balance FROM account_balances where account_id = $1",
        trans.from_account_id
    ).fetch_one(&pool).await.unwrap();
    let to_balance = sqlx::query_as!(
        BalanceResult,
        "SELECT account_id, balance FROM account_balances where account_id = $1",
        trans.to_account_id
    ).fetch_one(&pool).await.unwrap();

    let new_from_acc_balance = from_balance.balance - trans.clone().amount;
    let new_to_acc_balance = to_balance.balance + trans.clone().amount;

    if (new_from_acc_balance > BigDecimal::from(0)) && (new_to_acc_balance > BigDecimal::from(0)) {
        let new_from_balance = sqlx::query_scalar!(
            "UPDATE account_balances SET balance = $1 where account_id = $2 returning balance",
            new_from_acc_balance,
            from_balance.account_id
        ).fetch_one(&pool).await.unwrap();
        let new_to_balance = sqlx::query_scalar!(
            "UPDATE account_balances SET balance = $1 where account_id = $2 returning balance",
            new_to_acc_balance,
            to_balance.account_id
        ).fetch_one(&pool).await.unwrap();
        println!("from_balance = {new_from_balance}, to_balance = {new_to_balance}, amount = {}, failed lil bro", trans.amount.to_string());
    } else {
        println!("from_balance = {new_from_acc_balance}, to_balance = {new_to_acc_balance}, amount = {}, failed lil bro", trans.amount.to_string());
    }
    // TODO: else throw error

}

#[axum::debug_handler]
pub async fn create(State(state): State<state::AppState>, Json(req): Json<CreateTransReq>) {
    let pool = state.db;

    sqlx::query_scalar!(
        "INSERT INTO transactions (from_account_id, to_account_id, amount ) values ($1, $2, $3) returning id",
        req.from_account_id,
        req.to_account_id,
        req.amount,
    ).fetch_one(&pool).await.unwrap();

    update_balance(req, pool).await;

}

pub async fn get_all(State(state): State<state::AppState>) -> Json<Vec<Transaction>> {
    let pool = state.db;

    let res = sqlx::query_as!(
        Transaction,
        "SELECT * FROM transactions;"
    ).fetch_all(&pool)
     .await
     .unwrap();

    Json(res)
}

pub async fn query(State(state): State<state::AppState>) -> Json<Transaction> {
    let pool = state.db;

    let res = sqlx::query_as!(
        Transaction,
        "SELECT * FROM transactions",
    ).fetch_one(&pool).await.unwrap();

    Json(res)
}
