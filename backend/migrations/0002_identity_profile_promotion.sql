CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE TABLE crowd_state (singleton boolean PRIMARY KEY DEFAULT true CHECK(singleton), document jsonb NOT NULL);
INSERT INTO crowd_state VALUES (true, '{"users":{},"rules":[],"codes":{},"cases":[],"audit":[],"idempotency":{}}');
CREATE TABLE crowd_identities (platform_id text PRIMARY KEY, user_id text NOT NULL UNIQUE);
CREATE TABLE crowd_sessions (token_hash text PRIMARY KEY, user_id text NOT NULL, expires_at timestamptz NOT NULL);
CREATE TABLE crowd_files (id text PRIMARY KEY DEFAULT encode(gen_random_bytes(18),'hex'), owner_id text NOT NULL, purpose text NOT NULL CHECK(purpose IN ('avatar','wechat_receipt')), mime text NOT NULL, content bytea NOT NULL, created_at timestamptz NOT NULL DEFAULT now());
-- Business document updates, audit and idempotency commit together under row lock.
-- Sensitive tables are accessible only to the API database role; never expose database credentials to clients.

CREATE TABLE crowd_web_tickets (ticket_hash text PRIMARY KEY, user_id text NOT NULL, expires_at timestamptz NOT NULL);

CREATE TABLE crowd_code_jobs (user_id text PRIMARY KEY, status text NOT NULL CHECK(status IN ('pending','completed')), attempts integer NOT NULL DEFAULT 0, created_at timestamptz NOT NULL DEFAULT now());
CREATE TABLE crowd_code_ownership (code text PRIMARY KEY, owner_id text NOT NULL, product_id text, task_id text, active boolean NOT NULL, version bigint NOT NULL);
CREATE UNIQUE INDEX crowd_one_active_code_per_target ON crowd_code_ownership(owner_id,product_id,task_id) NULLS NOT DISTINCT WHERE active;
