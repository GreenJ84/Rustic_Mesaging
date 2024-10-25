use std::any::TypeId;
use std::sync::LazyLock;
use gloo::net::http::{Method, Request, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use crate::utils::get_auth_token;


pub fn api_url() -> String {
    option_env!("SERVER_URL")
        .unwrap_or("http://localhost:8000")
        .to_string()
}

pub async fn api_head(extension: String) -> bool{
    RequestBuilder::new(
        &format!("{}/{}", api_url(), extension)
    )
    .method(Method::HEAD)
    .header("Authorization", &get_auth_token())
    .send()
    .await
    .is_ok()
}

#[derive(Debug)]
pub enum PostResponse<T> {
    NonResponse(T),
    Response(T, Response),
    OnlyResponse(Response)
}

pub async fn api_post<T: DeserializeOwned + 'static>(extension: String, data: Vec<(String, String)>, need_response: bool) -> Result<PostResponse<T>, ()> {
    let result = Request::post(
        &format!("{}/{}", api_url(), extension)
    )
    .header("Content-Type", "application/x-www-form-urlencoded")
    .header("Authorization", &get_auth_token())
    .body(data.iter()
        .map(|(key, value)| format!("{}={}", key, urlencoding::encode(value)))
        .collect::<Vec<String>>()
        .join("&")
    )
    .unwrap()
    .send()
    .await;

    match result {
        Ok(response) => {
            if response.ok() {
                if TypeId::of::<T>() == TypeId::of::<()>(){
                    return Ok(PostResponse::OnlyResponse(response))
                }
                if let Ok(entity) = response.json::<T>().await
                    .map_err(|err| {
                        log::error!("Failed to parse response: {:?}", err);
                        ()
                    }) {
                    if need_response {
                        Ok(PostResponse::Response(entity, response))
                    } else {
                        Ok(PostResponse::NonResponse(entity))
                    }
                } else {
                    return Ok(PostResponse::OnlyResponse(response))
                }
            } else {
                log::error!("Request failed with status: {}", response.status());
                Ok(PostResponse::OnlyResponse(response))
            }
        }
        Err(err) => { log::error!("Failed to send request: {:?}", err); Err(()) }
    }
}

pub async fn api_get<T: DeserializeOwned>(extension: String) -> Result<T, ()> {
    let result =
        Request::get(
            &format!("{}/{}", api_url(), extension)
        )
            .header("Authorization", &get_auth_token())
            .send()
            .await;

    match result {
        Ok(response) => {
            if response.ok() {
                 response.json::<T>().await
                    .map_err(|err| {
                         log::error!("Failed to parse response: {:?}", err);
                         ()
                     })
            } else {
                log::error!("Failed to fetch servers: {}", response.status());
                Err(())
            }
        }
        Err(err) => {
            log::error!("Request failed: {:?}", err);
            Err(())
        },
    }
}

pub async fn api_put<T: DeserializeOwned>(extension: String, data: Vec<(String, String)>) -> Result<T, ()> {
    let result = Request::put(
            &format!("{}/{}", api_url(), extension)
        )
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Authorization", &get_auth_token())
        .body(data.iter()
            .map(|(key, value)| format!("{}={}", key, urlencoding::encode(value)))
            .collect::<Vec<String>>()
            .join("&")
        )
        .unwrap()
        .send()
        .await;

        match result {
            Ok(response) => {
                if response.ok() {
                    response.json::<T>().await
                        .map_err(|err| {
                            log::error!("Failed to parse response: {:?}", err);
                            ()
                        })
                } else { log::error!("Request failed with status: {}", response.status()); Err(()) }
            }
            Err(err) => { log::error!("Failed to send request: {:?}", err); Err(()) }
        }
}

pub async fn api_delete(extension: String) -> Result<(), ()> {
    let request = Request::delete(
            &format!("{}/{}", api_url(), extension)
        )
        .header("Authorization", &get_auth_token())
        .send()
        .await;

    match request {
        Ok(_response) => {
            Ok(())
        }
        Err(err) => {
            log::error!("Request failed: {:?}", err);
            Err(())
        },
    }
}