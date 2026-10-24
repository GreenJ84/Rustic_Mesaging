CREATE TABLE friend_request (
    sender_id INT NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    receiver_id INT NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    note TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT friend_request_members_must_differ
        CHECK (sender_id <> receiver_id),

    CONSTRAINT friend_request_pkey
        PRIMARY KEY (sender_id, receiver_id)
);

CREATE INDEX idx_receiver ON friend_request(receiver_id);
CREATE UNIQUE INDEX friend_request_pair_unique
ON friend_request (
    LEAST(sender_id, receiver_id),
    GREATEST(sender_id, receiver_id)
);

CREATE OR REPLACE FUNCTION already_friend_protection()
RETURNS TRIGGER AS $$
BEGIN
    -- Check if already friends
    IF EXISTS (
        SELECT 1
        FROM friend
        WHERE member_id = LEAST(NEW.receiver_id, NEW.sender_id)
            AND friend_id = GREATEST(NEW.receiver_id, NEW.sender_id)
    ) THEN
        RAISE EXCEPTION
            USING
                ERRCODE = '23505',
                CONSTRAINT = 'friend_request_already_friend',
                MESSAGE = 'Members are already friends';
    END IF;

    -- If no friendship exists, allow the friend request to be inserted
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_already_friend_protection
BEFORE INSERT ON friend_request
FOR EACH ROW
EXECUTE FUNCTION already_friend_protection();
