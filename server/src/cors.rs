use std::collections::HashSet;

use rocket::http;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions, Method};

pub fn create_cors(allowed_origins: &Vec<String>) -> CorsOptions {
    let allowed_origins = AllowedOrigins::some_exact(allowed_origins);
    CorsOptions::default()
        .allowed_origins(allowed_origins)
        .allowed_methods(
            [
                http::Method::Get,
                http::Method::Head,
                http::Method::Post,
                http::Method::Put,
                http::Method::Patch,
                http::Method::Delete,
            ]
            .into_iter()
            .map(|m| Method::from(m))
            .collect(),
        )
        .allowed_headers(AllowedHeaders::some(&[
            "Authorization",
            "Content-Type",
            "Administration-Key",
        ])) // Allow all headers
        .allow_credentials(true)
}
