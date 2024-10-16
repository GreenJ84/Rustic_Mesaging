CREATE TABLE member_chat (
    chat_id INT NOT NULL REFERENCES chat(id) ON DELETE CASCADE,
    member_id INT NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,


    PRIMARY KEY(chat_id, member_id)
);
CREATE INDEX member_chats ON member_chat(member_id);
CREATE INDEX chat_members ON member_chat(chat_id);
