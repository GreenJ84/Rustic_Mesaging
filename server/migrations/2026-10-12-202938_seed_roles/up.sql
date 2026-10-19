INSERT INTO system_role (name, authority_level, system_managed) VALUES
    ('admin', 50, TRUE),
    ('moderator', 40, TRUE),
    ('user', 30, TRUE),
    ('guest', 20, TRUE),
    ('suspended', 10, TRUE),
    ('banned', 0, TRUE);

INSERT INTO member_system_role (member_id, role_id, assigned_by)
SELECT member.id, system_role.id, member.id AS assigner
FROM member
CROSS JOIN system_role
WHERE system_role.name = 'user'
ON CONFLICT DO NOTHING;

INSERT INTO member_system_role (member_id, role_id, assigned_by)
SELECT member.id, system_role.id, member.id AS assigner
FROM member
CROSS JOIN system_role
WHERE system_role.name = 'admin'
  AND member.is_admin = TRUE
ON CONFLICT DO NOTHING;
