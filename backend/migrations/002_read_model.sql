CREATE TABLE IF NOT EXISTS read_request_list_item (
    request_id VARCHAR(128) PRIMARY KEY,
    asset_id VARCHAR(128) NOT NULL,
    asset_title VARCHAR(512) NOT NULL,
    asset_location VARCHAR(512) NOT NULL,
    description TEXT NOT NULL,
    priority VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL,
    assignee VARCHAR(128),
    open_escalation_count INT NOT NULL DEFAULT 0,
    work_order_count INT NOT NULL DEFAULT 0,
    overdue BOOLEAN NOT NULL DEFAULT FALSE,
    sla_deadline_epoch_sec BIGINT NOT NULL,
    created_at_epoch_sec BIGINT NOT NULL,
    owner_user_id VARCHAR(64) NOT NULL
);

CREATE TABLE IF NOT EXISTS read_request_detail (
    request_id VARCHAR(128) PRIMARY KEY,
    asset_id VARCHAR(128) NOT NULL,
    asset_title VARCHAR(512) NOT NULL,
    asset_location VARCHAR(512) NOT NULL,
    description TEXT NOT NULL,
    priority VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL,
    assignee VARCHAR(128),
    work_order_status VARCHAR(32),
    open_escalation_count INT NOT NULL DEFAULT 0,
    latest_escalation_reason TEXT,
    overdue BOOLEAN NOT NULL DEFAULT FALSE,
    sla_deadline_epoch_sec BIGINT NOT NULL,
    created_at_epoch_sec BIGINT NOT NULL,
    owner_user_id VARCHAR(64) NOT NULL
);

CREATE TABLE IF NOT EXISTS read_work_order_item (
    work_order_id VARCHAR(128) PRIMARY KEY,
    request_id VARCHAR(128) NOT NULL,
    assignee VARCHAR(128),
    assignee_name VARCHAR(512),
    status VARCHAR(32) NOT NULL,
    owner_user_id VARCHAR(64) NOT NULL
);

CREATE TABLE IF NOT EXISTS read_projection_error (
    id UUID PRIMARY KEY,
    event_id UUID,
    event_type VARCHAR(128) NOT NULL,
    entity_id VARCHAR(128) NOT NULL,
    error_message TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at_epoch_sec BIGINT NOT NULL
);
