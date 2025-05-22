-- Add down migration script here
DROP TRIGGER IF EXISTS users_audit_trigger ON users;
DROP TRIGGER IF EXISTS accounts_audit_trigger ON accounts;
DROP TRIGGER IF EXISTS account_balances_audit_trigger ON account_balances;
DROP TRIGGER IF EXISTS transactions_audit_trigger ON transactions;
DROP FUNCTION IF EXISTS audit_trigger_function();
