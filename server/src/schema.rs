// @generated automatically by Diesel CLI.

diesel::table! {
    auth_session (id) {
        id -> Uuid,
        family_id -> Uuid,
        member_id -> Int4,
        refresh_token_hash -> Text,
        refresh_expires_at -> Timestamp,
        user_agent -> Nullable<Text>,
        ip_address -> Nullable<Text>,
        created_at -> Timestamp,
        consumed_at -> Nullable<Timestamp>,
        replaced_by -> Nullable<Uuid>,
        revoked_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    channel (id) {
        id -> Int4,
        server_id -> Int4,
        name -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    chat (id) {
        id -> Int4,
        name -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    friend (member_id, friend_id) {
        member_id -> Int4,
        friend_id -> Int4,
        created_at -> Timestamp,
    }
}

diesel::table! {
    friend_request (sender_id, receiver_id) {
        sender_id -> Int4,
        receiver_id -> Int4,
        note -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    member (id) {
        id -> Int4,
        username -> Text,
        password_hash -> Text,
        email -> Text,
        avatar -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    member_chat (chat_id, member_id) {
        chat_id -> Int4,
        member_id -> Int4,
        joined_at -> Timestamp,
    }
}

diesel::table! {
    member_system_role (member_id, role_id) {
        member_id -> Int4,
        role_id -> Int4,
        assigned_by -> Nullable<Int4>,
        assigned_at -> Timestamp,
    }
}

diesel::table! {
    message (id) {
        id -> Int4,
        content -> Text,
        sender_id -> Int4,
        chat_id -> Int4,
        created_at -> Timestamp,
    }
}

diesel::table! {
    post (id) {
        id -> Int4,
        content -> Text,
        author_id -> Int4,
        channel_id -> Int4,
        created_at -> Timestamp,
    }
}

diesel::table! {
    server (id) {
        id -> Int4,
        name -> Text,
        description -> Text,
        icon -> Text,
        owner_id -> Int4,
        created_at -> Timestamp,
    }
}

diesel::table! {
    server_membership (member_id, server_id) {
        member_id -> Int4,
        server_id -> Int4,
        joined_at -> Timestamp,
    }
}

diesel::table! {
    system_role (id) {
        id -> Int4,
        name -> Text,
        authority_level -> Int4,
        system_managed -> Bool,
    }
}

diesel::joinable!(auth_session -> member (member_id));
diesel::joinable!(channel -> server (server_id));
diesel::joinable!(member_chat -> chat (chat_id));
diesel::joinable!(member_chat -> member (member_id));
diesel::joinable!(member_system_role -> system_role (role_id));
diesel::joinable!(message -> chat (chat_id));
diesel::joinable!(message -> member (sender_id));
diesel::joinable!(post -> channel (channel_id));
diesel::joinable!(post -> member (author_id));
diesel::joinable!(server -> member (owner_id));
diesel::joinable!(server_membership -> member (member_id));
diesel::joinable!(server_membership -> server (server_id));

diesel::allow_tables_to_appear_in_same_query!(
    auth_session,
    channel,
    chat,
    friend,
    friend_request,
    member,
    member_chat,
    member_system_role,
    message,
    post,
    server,
    server_membership,
    system_role,
);
