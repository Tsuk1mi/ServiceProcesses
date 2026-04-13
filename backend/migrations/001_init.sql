CREATE TABLE IF NOT EXISTS app_user (
    id UUID PRIMARY KEY,
    subject_id UUID NOT NULL,
    username VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_user_role (
    user_id UUID NOT NULL REFERENCES app_user(id) ON DELETE CASCADE,
    role VARCHAR(64) NOT NULL,
    PRIMARY KEY (user_id, role)
);

CREATE TABLE IF NOT EXISTS asset (
    id VARCHAR(128) PRIMARY KEY,
    kind VARCHAR(512) NOT NULL,
    title VARCHAR(512) NOT NULL,
    location VARCHAR(512) NOT NULL,
    state VARCHAR(32) NOT NULL,
    owner_user_id VARCHAR(64) NOT NULL
);

CREATE TABLE IF NOT EXISTS service_request (
    id VARCHAR(128) PRIMARY KEY,
    asset_id VARCHAR(128) NOT NULL REFERENCES asset(id),
    description TEXT NOT NULL,
    priority VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL,
    sla_minutes INT NOT NULL,
    created_at_epoch_sec BIGINT NOT NULL,
    owner_user_id VARCHAR(64) NOT NULL
);

CREATE TABLE IF NOT EXISTS work_order (
    id VARCHAR(128) PRIMARY KEY,
    request_id VARCHAR(128) NOT NULL REFERENCES service_request(id),
    assignee VARCHAR(128),
    status VARCHAR(32) NOT NULL,
    owner_user_id VARCHAR(64) NOT NULL
);

CREATE TABLE IF NOT EXISTS escalation (
    id VARCHAR(128) PRIMARY KEY,
    request_id VARCHAR(128) NOT NULL REFERENCES service_request(id),
    reason TEXT NOT NULL,
    state VARCHAR(32) NOT NULL,
    owner_user_id VARCHAR(64) NOT NULL
);

CREATE TABLE IF NOT EXISTS technician (
    id VARCHAR(128) PRIMARY KEY,
    full_name VARCHAR(512) NOT NULL,
    skills JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    owner_user_id VARCHAR(64) NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_record (
    id VARCHAR(128) PRIMARY KEY,
    request_id VARCHAR(128),
    entity VARCHAR(128) NOT NULL,
    action VARCHAR(128) NOT NULL,
    actor_role VARCHAR(128) NOT NULL,
    actor_id VARCHAR(128),
    details TEXT NOT NULL,
    created_at_utc VARCHAR(64) NOT NULL,
    owner_user_id VARCHAR(64) NOT NULL
);

CREATE TABLE IF NOT EXISTS analytics_snapshot (
    singleton VARCHAR(1) PRIMARY KEY DEFAULT 'x' CHECK (singleton = 'x'),
    payload JSONB NOT NULL,
    updated_at_epoch_sec BIGINT NOT NULL DEFAULT 0
);
