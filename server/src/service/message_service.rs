use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods, OptionalExtension};
use diesel::result::Error;

use crate::db::DbConn;
use crate::service::CrudOps;
use crate::models::message::{NewMessage, Message};
use crate::schema::message::dsl;


pub struct MessageService;
impl CrudOps<NewMessage, Message> for MessageService{
    fn create(conn: &mut DbConn, new_message: NewMessage) -> Result<Message, Error> {
        diesel::insert_into(dsl::message)
            .values(&new_message)
            .get_result::<Message>(conn)
    }

    fn read(conn: &mut DbConn, id: i32) -> Result<Message, Error> {
        let message: Option<Message> = dsl::message
            .find(id)
            .first::<Message>(conn)
            .optional()
            .map_err(|e| e)?;

        match message {
            Some(message) => Ok(message),
            None => Err(Error::NotFound)
        }
    }

    fn update(conn: &mut DbConn, id: i32, entity: NewMessage) -> Result<Message, Error> {
        let updated_fields = (
            dsl::content.eq(entity.content().to_owned()),
        );

        diesel::update(dsl::message.find(id))
            .set(updated_fields)
            .get_result::<Message>(conn)
    }

    fn delete(conn: &mut DbConn, id: i32) -> Result<usize, Error> {
        diesel::delete(dsl::message.find(id)).execute(conn)
    }
}
