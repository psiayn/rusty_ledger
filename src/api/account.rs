use std::fmt;

use axum::{extract::{Query, State}, Json};
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

pub async fn check_balance(State(state): State<state::AppState>, Query(req): Query<AccountBalanceReq>) -> Json<AccountBalance> {
    let pool = state.db;

    let user = sqlx::query_as!(
        AccountBalance,
        "SELECT account_id, balance FROM account_balances where account_id = $1",
        req.account_id
    ).fetch_one(&pool).await.unwrap();
    Json(user)
}

pub async fn update_balance(State(state): State<state::AppState>, Json(req): Json<AccountBalance>) -> Json<AccountBalance> {
    let pool = state.db;
    let balance = sqlx::query_scalar!(
        "UPDATE account_balances SET balance = $1 where account_id = $2 returning balance",
        req.balance,
        req.account_id
    ).fetch_one(&pool).await.unwrap();
    let res = AccountBalance {
        account_id: req.account_id,
        balance: balance
    };

    Json(res)
}
