//! CORS policy construction for the HTTP application.

use rocket::http;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions, Method};

/// Builds the CORS policy from configured exact allowed origins.
pub fn create_cors(allowed_origins: &[String]) -> CorsOptions {
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
        ]))
        .allow_credentials(true)
}

#[cfg(test)]
mod tests {
    use super::create_cors;

    #[test]
    fn builds_cors_configuration_for_allowed_origins() {
        let origins = vec!["http://localhost:3000".to_string()];
        assert!(create_cors(&origins).to_cors().is_ok());
    }

    #[test]
    fn builds_cors_configuration_without_allowed_origins() {
        assert!(create_cors(&[]).to_cors().is_ok());
    }
}
