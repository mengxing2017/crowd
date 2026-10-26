CREATE TABLE crowd_staff_accounts (
    username text PRIMARY KEY,
    user_id text NOT NULL UNIQUE,
    password_hash text NOT NULL,
    failed_attempts integer NOT NULL DEFAULT 0,
    blocked_until timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);
