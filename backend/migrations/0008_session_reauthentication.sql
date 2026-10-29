CREATE TABLE crowd_session_reauthentication (
 token_hash text PRIMARY KEY REFERENCES crowd_sessions(token_hash) ON DELETE CASCADE,
 verified_at timestamptz NOT NULL DEFAULT now(),
 policy_version bigint NOT NULL
);
