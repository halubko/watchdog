-- Add migration script here
CREATE TYPE check_status as ENUM ('success', 'unexpected_status', 'fail');
CREATE TYPE failure_reason as ENUM ('timeout', 'connection_refused', 'dns_resolution', 'other');

CREATE TABLE endpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    url VARCHAR NOT NULL,
    interval_seconds INTEGER NOT NULL,
    timeout INTEGER NULL,
    expected_status INTEGER NOT NULL
);

CREATE TABLE check_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint_id UUID NOT NULL REFERENCES endpoints(id) ON DELETE CASCADE,
    date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status_code INTEGER NULL,
    latency_ms INTEGER NULL,
    failure_reason failure_reason NULL,
    failure_message TEXT NULL,
    status check_status NOT NULL
);

CREATE INDEX idx_endpoint_id_date ON check_results(endpoint_id, date);