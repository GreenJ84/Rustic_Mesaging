CREATE TABLE auth_session (
    id UUID PRIMARY KEY,
    family_id UUID NOT NULL,
    member_id INT NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    refresh_token_hash TEXT NOT NULL UNIQUE,
    refresh_expires_at TIMESTAMP NOT NULL,
    user_agent TEXT,
    ip_address TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    consumed_at TIMESTAMP DEFAULT NULL,
    replaced_by UUID REFERENCES auth_session(id) NULL,
    revoked_at TIMESTAMP DEFAULT NULL
);
CREATE INDEX auth_session_family_id_idx ON auth_session (family_id);
CREATE INDEX auth_session_member_id_idx ON auth_session (member_id);