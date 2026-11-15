pub(super) mod error;
pub(crate) mod models;
pub(crate) mod repository;
pub(crate) mod routes;
pub(crate) mod service;

pub(crate) fn get_member_routes() -> Vec<rocket::Route> {
    let mut routes = routes::routes();
    routes.extend(crate::server::routes::member_routes());
    routes
}

#[cfg(test)]
mod tests {
    use rocket::http::Method;

    use super::get_member_routes;

    #[test]
    fn member_route_collection_includes_member_owned_server_resources() {
        let routes = get_member_routes();
        assert!(routes.iter().any(|route| {
            route.method == Method::Get && route.uri.to_string() == "/servers?<page>"
        }));
        assert_eq!(routes.len(), 8);
    }
}
