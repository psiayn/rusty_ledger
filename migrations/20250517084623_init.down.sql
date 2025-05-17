-- Add down migration script here
-- drop triggers
DROP TRIGGER IF EXISTS trg_update_balances ON transactions;

-- drop functions
DROP FUNCTION IF EXISTS update_account_balances;

-- drop tables
DROP TABLE IF EXISTS audit_logs CASCADE;
DROP TABLE IF EXISTS exchange_rates CASCADE;
DROP TABLE IF EXISTS transactions CASCADE;
DROP TABLE IF EXISTS account_balances CASCADE;
DROP TABLE IF EXISTS accounts CASCADE;
DROP TABLE IF EXISTS users CASCADE;
