use rocket::launch;
use rustic_messaging::app;

#[launch]
pub fn rocket() -> _ {
    app::build()
}
