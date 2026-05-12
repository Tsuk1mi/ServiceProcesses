CREATE TABLE IF NOT EXISTS auth_session (
    session_id UUID PRIMARY KEY,
    user_subject_id UUID NOT NULL,
    refresh_token_hash TEXT NOT NULL,
    client_kind VARCHAR(64) NOT NULL,
    created_at_epoch_sec BIGINT NOT NULL,
    expires_at_epoch_sec BIGINT NOT NULL,
    revoked_at_epoch_sec BIGINT
);

CREATE INDEX IF NOT EXISTS idx_auth_session_user_subject_id
    ON auth_session (user_subject_id);

CREATE INDEX IF NOT EXISTS idx_auth_session_expires_at
    ON auth_session (expires_at_epoch_sec);
