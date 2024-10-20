use chrono::NaiveDateTime;
use wasm_bindgen::JsValue;
use web_sys::Storage;

pub(crate) mod api_requests;

pub fn get_local_storage() -> Storage{
    web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .unwrap()
}

pub fn get_auth_token() -> String {
    get_local_storage()
        .get_item("jwt_token")
        .unwrap()
        .unwrap_or(String::new())
}

pub fn set_auth_token(token: String) -> Result<(), JsValue>{
    get_local_storage()
        .set_item("jwt_token", &token)
}

pub fn format_date(datetime: &NaiveDateTime) -> String{
    return datetime.format("%m-%d-%Y %H:%M:%S").to_string();
}
