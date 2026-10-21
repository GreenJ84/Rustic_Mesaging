use rocket::{Build, Rocket};

use crate::config::AppConfig;

fn build() -> Rocket<Build> {
    let config = AppConfig::load().expect("application configuration must be valid");

    rocket::build()
}
