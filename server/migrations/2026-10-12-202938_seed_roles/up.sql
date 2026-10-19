INSERT INTO system_role (name, authority_level, system_managed) VALUES
    ('admin', 50, TRUE),
    ('moderator', 40, TRUE),
    ('user', 30, TRUE),
    ('guest', 20, TRUE),
    ('suspended', 10, TRUE),
    ('banned', 0, TRUE);

INSERT INTO member_system_role (member_id, role_id, assigned_by)
SELECT member.id, system_role.id AS assigner, (SELECT id FROM member WHERE username = 'SYSTEM' LIMIT 1)
FROM member
CROSS JOIN system_role
WHERE system_role.name = 'admin'
  AND member.username = 'GreenJ'
ON CONFLICT DO NOTHING;

INSERT INTO member_system_role (member_id, role_id, assigned_by)
SELECT
    m.id,
    (SELECT sr.id FROM system_role sr WHERE sr.name = 'user'),
    (SELECT id FROM member WHERE username = 'SYSTEM' LIMIT 1)
FROM member m
ON CONFLICT DO NOTHING;