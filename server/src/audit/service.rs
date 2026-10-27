use crate::db::{DbConn, DbError};
use crate::member::models::MemberProfile;
use crate::models::report::{ActivityReportItem, ServerMemberReportItem};
use crate::schema::{member, server_membership};
use chrono::NaiveDateTime;
use diesel::{
    sql_query, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl,
    SelectableHelper,
};

pub struct ReportService;

impl ReportService {
    pub fn generate_activity_report(conn: &mut DbConn, member_id: i32) -> Vec<ActivityReportItem> {
        sql_query(
            "
            -- Post creation activity
            SELECT 'Post'::text as entity, CONCAT('Content: ', p.content::text)::text as description, p.created_at::timestamp as timestamp
            FROM post p
            WHERE p.author_id = $1

            UNION

            -- Message creation activity
            SELECT 'Message'::text as entity, CONCAT('Content: ', m.content::text)::text as description, m.created_at::timestamp as timestamp
            FROM message m
            WHERE m.sender_id = $1

            UNION

            -- Server creation activity
            SELECT 'Server'::text as entity, CONCAT('Created: ', s.name::text)::text as description, s.created_at::timestamp as timestamp
            FROM server s
            WHERE s.owner_id = $1

            UNION

            -- Channel creation activity
            SELECT 'Channel'::text as entity, CONCAT('Created: ', c.name::text)::text as description, c.created_at::timestamp as timestamp
            FROM channel c
            WHERE c.server_id IN (
                SELECT sm.server_id FROM server_membership sm WHERE sm.member_id = $1
            )

            UNION

            -- Friend request activity
            SELECT 'Friend Request'::text as entity, CONCAT('Sent friend request to ', m.username::text)::text as description, fr.created_at::timestamp as timestamp
            FROM friend_request fr
            JOIN member m ON m.id = fr.receiver_id
            WHERE fr.sender_id = $1

            UNION

            -- Friendship activity
            SELECT 'Friendship'::text as entity,
               CASE
                    WHEN f.member_id = $1 THEN CONCAT('Became friends with ', m.username::text)::text
                    WHEN f.friend_id = $1 THEN CONCAT('Became friends with ', m2.username::text)::text
                END as description,
                f.created_at::timestamp as timestamp
            FROM friend f
            LEFT JOIN member m
                ON m.id = f.friend_id
            LEFT JOIN member m2
                ON m2.id = f.member_id
            WHERE f.member_id = $1 OR f.friend_id = $1
            ORDER BY timestamp DESC
            "
        )
            .bind::<diesel::sql_types::Integer, _>(member_id)
            .get_results::<ActivityReportItem>(conn)
            .map_err(|e| format!("{}", e))
            .unwrap()
    }

    pub fn generate_membership_report(
        conn: &mut DbConn,
        server_id: i32,
    ) -> Result<Vec<ServerMemberReportItem>, DbError> {
        let rows = member::table
            .inner_join(server_membership::table.on(member::id.eq(server_membership::member_id)))
            .filter(server_membership::server_id.eq(server_id))
            .order(server_membership::joined_at.desc())
            .select((MemberProfile::as_select(), server_membership::joined_at))
            .load::<(MemberProfile, NaiveDateTime)>(conn)?;

        Ok(rows
            .into_iter()
            .map(|(member, timestamp)| ServerMemberReportItem { member, timestamp })
            .collect())
    }
}
