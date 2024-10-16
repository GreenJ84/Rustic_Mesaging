use gloo::net::http::Request;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, MouseEvent, SubmitEvent, window};
use yew::{Callback, function_component, Html, html, Properties, use_context, use_node_ref, use_state};
use yew_router::hooks::{use_location, use_navigator};
use crate::contexts::member_context::{MemberDispatch, get_servers, TMemberContext};
use crate::models::server::{MultiServer, Server};
use crate::utils::auth_token;
use crate::comps::modal::{Modal, toggle_modal};
use crate::views::home::HomeRoute;
use crate::views::home::me::MeRoute;
use crate::contexts::server_context::{ServerDispatch, TServerContext};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub(crate) callback: Callback<MouseEvent>
}

#[function_component(EditServerForm)]
pub fn edit_server_modal(Props { callback }: &Props) -> Html {
    let app_ctx = use_context::<TMemberContext>().unwrap();
    let server_ctx = use_context::<TServerContext>().unwrap();
    let navigation = use_navigator().unwrap();
    let location = use_location().unwrap();

    let name_ref = use_node_ref();
    let description_ref = use_node_ref();

    let on_submit = {
        let app_ctx = app_ctx.clone();
        let server_ctx = server_ctx.clone();
        let callback = callback.clone();

        let name_ref = name_ref.clone();
        let description_ref = description_ref.clone();
        let location = location.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let app_ctx = app_ctx.clone();
            let server_ctx = server_ctx.clone();
            let callback = callback.clone();
            let location = location.clone();

            let name = name_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            let description = description_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            let mut form_data = vec![
                ("name", name.clone()),
                ("description", description.clone()),
                ("icon", String::new()),
                ("owner_id", app_ctx.member.id.clone().to_string())
            ];
            wasm_bindgen_futures::spawn_local(async move {
                let request = Request::put(
                    &format!("http://localhost:8000/server/{}",
                            server_ctx.current_server.id
                    ))
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .header("Authorization", &auth_token())
                    .body(form_data.iter()
                        .map(|(key, value)| format!("{}={}", key, urlencoding::encode(value)))
                        .collect::<Vec<String>>()
                        .join("&")
                    )
                    .unwrap()
                    .send()
                    .await;

                match request {
                    Ok(response) => {
                        if response.ok() {
                            let text = response.text().await.unwrap_or_default();
                            match serde_json::from_str::<Server>(&text) {
                                Ok(server) => {
                                    app_ctx.dispatch(MemberDispatch::UpdateServers(get_servers().await.unwrap()));
                                    toggle_modal("modal_overlay");
                                    callback.emit(MouseEvent::new("submit").unwrap());
                                    server_ctx.dispatch(ServerDispatch::UpdateServer(server.clone()));
                                    // location.refresh();
                                }
                                Err(_) => { log::error!("Failed to parse response"); }
                            }
                        } else { log::error!("Request failed with status: {}", response.status()); }
                    }
                    Err(err) => { log::error!("Failed to send request: {:?}", err); }
                }
            });
        })
    };

    html! {
        <Modal
            modal_class={"edit_server_modal"}
            button_class={"edit_server"}
            button_icon={html!(
                {"Edit Server"}
            )}
        >
            <>
                <h2>{ "Edit Server" }</h2>
                <form onsubmit={on_submit} >
                    <label for="name">
                        { "Name:" }
                        <input
                            ref={name_ref}
                            type="text"
                            id="name"
                            name="name"
                            value={server_ctx.current_server.name.clone()}
                            required=true
                        />
                    </label>
                    <br/>
                    <label for="description">
                        { "Description:" }
                        <textarea
                            ref={description_ref}
                            id="description"
                            name="description"
                            value={server_ctx.current_server.description.clone()}
                            required=true
                        ></textarea>
                    </label>
                    <br/>
                    <button type="submit" style="margin-top: 10px;">{ "Create" }</button>
                </form>
            </>
        </Modal>
    }
}