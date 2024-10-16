CREATE TABLE friend (
    member_id INT NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    friend_id INT NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,


    PRIMARY KEY (member_id, friend_id)
);

CREATE INDEX idx_member ON friend(member_id);
CREATE INDEX idx_friend ON friend(friend_id);

-- Create the trigger function to enforce friendship order
CREATE OR REPLACE FUNCTION enforce_friendship_order_func()
RETURNS TRIGGER AS $$
    DECLARE temp INTEGER;

    BEGIN
        IF NEW.member_id > NEW.friend_id THEN
            temp := NEW.member_id;
            NEW.member_id := NEW.friend_id;
            NEW.friend_id := temp;
        END IF;

        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

-- Create the trigger to call the function before each insert
CREATE TRIGGER check_friendship
BEFORE INSERT ON friend
FOR EACH ROW
EXECUTE FUNCTION enforce_friendship_order_func();