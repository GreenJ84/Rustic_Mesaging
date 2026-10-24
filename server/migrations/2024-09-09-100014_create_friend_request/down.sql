-- This file should undo anything in `up.sql`
DROP TRIGGER IF EXISTS trigger_already_friend_protection ON friend_request;
DROP FUNCTION IF EXISTS already_friend_protection();
DROP INDEX IF EXISTS friend_request_pair_unique;
DROP INDEX IF EXISTS idx_receiver;
DROP TABLE IF EXISTS friend_request;