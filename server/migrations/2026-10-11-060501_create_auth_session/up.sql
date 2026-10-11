CREATE TABLE auth_session (
    id UUID PRIMARY KEY,
    family_id UUID NOT NULL,
    member_id INT NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    refresh_token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP NOT NULL,
    revoked_at TIMESTAMP,
    replaced_by UUID REFERENCES auth_session(id),
    user_agent TEXT,
    ip_address TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);