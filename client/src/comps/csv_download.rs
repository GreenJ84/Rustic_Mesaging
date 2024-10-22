use gloo::net::http::Request;
use wasm_bindgen::JsCast;
use web_sys::window;
use yew::prelude::*;
use crate::models::report::CsvDownload;
use crate::utils::api_requests::{api_get, API_URL};
use crate::utils::get_auth_token;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub(crate) url_extension: String
}
#[function_component(DownloadCsvButton)]
pub fn download_csv_button(Props {url_extension}: &Props) -> Html {
    let url_extension = url_extension.clone();
    let onclick = Callback::from(move |_| {
        let url_extension = url_extension.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let url_extension = url_extension.clone();
            let result = Request::get(
                &format!("{}/{}", API_URL, url_extension)
            )
                .header("Authorization", &get_auth_token())
                .send()
                .await;

            match result {
                Ok(response) => {
                    if response.ok() {
                        if let Ok(content) = response.text().await {
                            let window = window().unwrap();
                            let document = window.document().unwrap();

                            let a = document.create_element("a").unwrap();
                            a.set_attribute("href", &format!("data:text/csv;charset=utf-8,{}", content))
                                .unwrap();
                            a.set_attribute("download", "report.csv").unwrap();

                            let body = document.body().unwrap();
                            body.append_child(&a).unwrap();
                            a.dyn_ref::<web_sys::HtmlElement>().unwrap().click();
                            body.remove_child(&a).unwrap();
                        } else {
                            log::error!("Failed to parse csv response");
                        }
                    } else {
                        log::error!("Failed to fetch servers: {}", response.status());
                    }
                }
                Err(err) => {
                    log::error!("Request failed: {:?}", err);
                },
            }
        });
    });

    html! {
        <button {onclick}>{ "Download CSV" }</button>
    }
}
