pub(crate) mod api_requests;

pub fn auth_token() -> String{
    web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .unwrap()
        .get_item("jwt_token")
        .unwrap()
        .unwrap_or(String::new())
}
