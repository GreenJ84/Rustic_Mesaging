CREATE TABLE friend_request (
    sender_id INT NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    receiver_id INT NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    note TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (sender_id, receiver_id)
);

CREATE INDEX idx_requester ON friend_request(sender_id);
CREATE INDEX idx_receiver ON friend_request(receiver_id);

CREATE OR REPLACE FUNCTION resolve_to_friendship()
RETURNS TRIGGER AS $$
BEGIN
    -- Check if there is a reverse friend request
    IF EXISTS (
        SELECT 1
        FROM friend_request
        WHERE
            sender_id = NEW.receiver_id AND receiver_id = NEW.sender_id
    ) THEN
        -- Insert friend entry
        INSERT INTO friend (member_id, friend_id, created_at)
        VALUES (NEW.sender_id, NEW.receiver_id, NOW());

        -- Delete both friend requests (the current and the reverse)
        DELETE FROM friend_request
        WHERE
            (sender_id = NEW.sender_id AND receiver_id = NEW.receiver_id)
            OR (sender_id = NEW.receiver_id AND receiver_id = NEW.sender_id);

        -- No need to insert the new friend request, return NULL to skip insertion
        RETURN NULL;
    END IF;

    -- If no reverse request exists, allow the friend request to be inserted
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_resolve_to_friendship
BEFORE INSERT ON friend_request
FOR EACH ROW
EXECUTE FUNCTION resolve_to_friendship();