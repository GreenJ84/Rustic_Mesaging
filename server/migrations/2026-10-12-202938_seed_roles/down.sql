DELETE FROM member_system_role
WHERE role_id IN (
    SELECT id FROM system_role WHERE name IN ('admin', 'moderator', 'user', 'guest', 'suspended', 'banned')
);

DELETE FROM system_role WHERE name IN ('admin', 'moderator', 'user', 'guest', 'suspended', 'banned');
