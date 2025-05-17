-- Initialize Database

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY,
    full_name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Accounts table
CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_number TEXT UNIQUE NOT NULL,
    currency_code CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Balances table
CREATE TABLE account_balances (
    account_id UUID PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    balance NUMERIC(20, 4) NOT NULL DEFAULT 0.0,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Transactions table
CREATE TABLE transactions (
    id UUID PRIMARY KEY,
    from_account_id UUID REFERENCES accounts(id),
    to_account_id UUID REFERENCES accounts(id),
    amount NUMERIC(20, 4) NOT NULL CHECK (amount > 0),
    currency_code CHAR(3) NOT NULL,
    transaction_type TEXT NOT NULL CHECK (
        transaction_type IN ('deposit', 'withdrawal', 'transfer')
    ),
    status TEXT NOT NULL CHECK (
        status IN ('pending', 'authorized', 'settled', 'failed', 'reversed')
    ),
    reference TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    settled_at TIMESTAMPTZ
);

-- Reversal transactions reference the original one
ALTER TABLE transactions
ADD COLUMN reversed_transaction_id UUID REFERENCES transactions(id),
ADD CONSTRAINT no_self_reverse CHECK (
    reversed_transaction_id IS NULL OR reversed_transaction_id != id
);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    entity_type TEXT NOT NULL,            -- e.g., 'user', 'account', 'transaction'
    entity_id UUID NOT NULL,
    operation TEXT NOT NULL,              -- e.g., 'insert', 'update', 'delete'
    performed_by UUID REFERENCES users(id),
    before_state JSONB,
    after_state JSONB,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE exchange_rates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    base_currency CHAR(3) NOT NULL,
    quote_currency CHAR(3) NOT NULL,
    rate NUMERIC(20, 8) NOT NULL CHECK (rate > 0),
    effective_at TIMESTAMPTZ NOT NULL
);

CREATE FUNCTION update_account_balances() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.transaction_type = 'deposit' THEN
        UPDATE account_balances
        SET balance = balance + NEW.amount,
            updated_at = CURRENT_TIMESTAMP
        WHERE account_id = NEW.to_account_id;

    ELSIF NEW.transaction_type = 'withdrawal' THEN
        UPDATE account_balances
        SET balance = balance - NEW.amount,
            updated_at = CURRENT_TIMESTAMP
        WHERE account_id = NEW.from_account_id;

    ELSIF NEW.transaction_type = 'transfer' THEN
        UPDATE account_balances
        SET balance = balance - NEW.amount,
            updated_at = CURRENT_TIMESTAMP
        WHERE account_id = NEW.from_account_id;

        UPDATE account_balances
        SET balance = balance + NEW.amount,
            updated_at = CURRENT_TIMESTAMP
        WHERE account_id = NEW.to_account_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_balances
AFTER INSERT ON transactions
FOR EACH ROW
EXECUTE FUNCTION update_account_balances();
