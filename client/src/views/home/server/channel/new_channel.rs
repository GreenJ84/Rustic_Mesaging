use gloo::net::http::Request;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, MouseEvent, SubmitEvent, window};
use yew::{Callback, function_component, Html, html, use_context, use_node_ref, use_state};
use yew_router::hooks::use_navigator;
use crate::models::channel::Channel;
use crate::contexts::member_context::{MemberDispatch, get_servers, TMemberContext};
use crate::models::server::{MultiServer, Server};
use crate::utils::auth_token;
use crate::comps::modal::{Modal, toggle_modal};
use crate::views::home::HomeRoute;
use crate::views::home::me::MeRoute;
use crate::contexts::server_context::{get_server_channels, ServerDispatch, TServerContext};
use crate::views::home::server::ServerRoute;

#[function_component(NewChannelForm)]
pub fn new_channel_modal() -> Html {
    let app_ctx = use_context::<TMemberContext>().unwrap();
    let server_ctx = use_context::<TServerContext>().unwrap();
    let navigation = use_navigator().unwrap();

    let name_ref = use_node_ref();

    let on_submit = {
        let app_ctx = app_ctx.clone();
        let server_ctx = server_ctx.clone();
        let name_ref = name_ref.clone();
        let submit_nav = navigation.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let app_ctx = app_ctx.clone();
            let server_ctx = server_ctx.clone();
            let submit_nav = submit_nav.clone();

            let name = name_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            let mut form_data = vec![
                ("name", name.clone()),
                ("server_id", server_ctx.current_server.id.clone().to_string())
            ];
            wasm_bindgen_futures::spawn_local(async move {
                let app_ctx = app_ctx.clone();
                let server_ctx = server_ctx.clone();
                let request = Request::post("http://localhost:8000/channel")
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
                            match serde_json::from_str::<Channel>(&text) {
                                Ok(channel) => {
                                    server_ctx.dispatch(ServerDispatch::UpdateServerChannels(get_server_channels(server_ctx.current_server.id.clone()).await.unwrap()));
                                    server_ctx.dispatch(ServerDispatch::UpdateChannel(channel.clone()));
                                    toggle_modal("modal_overlay");
                                    submit_nav.push(&ServerRoute::Channel { server_id: server_ctx.current_channel.id, channel_id: channel.id })
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
            modal_class={"new_channel_modal"}
            button_class={"new_channel"}
            button_icon={html!(
                {"Create Channel"}
            )}
        >
            <>
                <h2>{ "Create Channel" }</h2>
                <form onsubmit={on_submit} >
                    <label for="name">
                        { "Name:" }
                        <input
                            ref={name_ref}
                            type="text"
                            id="name"
                            name="name"
                            required=true
                        />
                    </label>
                    <br/>
                    <button type="submit" style="margin-top: 10px;">{ "Create" }</button>
                </form>
            </>
        </Modal>
    }
}