use chrono::NaiveDateTime;
use diesel::{OptionalExtension, RunQueryDsl, sql_query};
use crate::db::DbConn;
use crate::models::report::{ActivityReportItem, ServerMemberReportItem};

pub struct ReportService;

impl ReportService {
    pub fn generate_activity_report(conn: &mut DbConn, member_id: i32) -> Vec<ActivityReportItem> {
        sql_query(
            "
            -- Post creation activity
            SELECT 'Post'::text as entity, p.content::text as description, p.created_at::timestamp as timestamp
            FROM post p
            WHERE p.author_id = $1

            UNION

            -- Message creation activity
            SELECT 'Message'::text as entity, m.content::text as description, m.created_at::timestamp as timestamp
            FROM message m
            WHERE m.sender_id = $1

            UNION

            -- Server creation activity
            SELECT 'Server'::text as entity, s.name::text as description, s.created_at::timestamp as timestamp
            FROM server s
            WHERE s.owner_id = $1

            UNION

            -- Channel creation activity
            SELECT 'Channel'::text as entity, c.name::text as description, c.created_at::timestamp as timestamp
            FROM channel c
            WHERE c.server_id IN (
                SELECT sm.server_id FROM server_membership sm WHERE sm.member_id = $1
            )

            UNION

            -- Friend request activity
            SELECT 'Friend Request'::text as entity, 'Sent friend request'::text as description, fr.created_at::timestamp as timestamp
            FROM friend_request fr
            WHERE fr.sender_id = $1

            UNION

            -- Friendship activity
            SELECT 'Friendship'::text as entity, 'Became friends with member'::text as description, f.created_at::timestamp as timestamp
            FROM friend f
            WHERE f.member_id = $1 OR f.friend_id = $1
            ORDER BY timestamp DESC
            "
        )
            .bind::<diesel::sql_types::Integer, _>(member_id)
            .get_results::<ActivityReportItem>(conn)
            .unwrap_or(Vec::new())
    }

    pub fn generate_membership_report(conn: &mut DbConn, server_id: i32) -> Vec<ServerMemberReportItem>{
        sql_query(
            "
            SELECT
                m.id as id,
                m.username as username,
                m.email as email,
                m.avatar as avatar,
                m.is_admin as is_admin,
                m.created_at as created_at,
                sm.joined_at as timestamp
            FROM
                member m
            JOIN
                server_membership sm
            ON
                m.id = sm.member_id
            WHERE
                sm.server_id = $1  -- Replace with the specific server_id you want to filter on.
            ORDER BY
                sm.joined_at DESC;
            "
        )
            .bind::<diesel::sql_types::Integer, _>(server_id)
            .get_results::<ServerMemberReportItem>(conn)
            .unwrap_or(Vec::new())
    }
}
