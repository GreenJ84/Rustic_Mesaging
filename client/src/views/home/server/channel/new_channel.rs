use gloo::net::http::Request;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, MouseEvent, SubmitEvent, window};
use yew::{Callback, function_component, Html, html, use_context, use_node_ref, use_state};
use yew_router::hooks::use_navigator;
use crate::models::channel::Channel;
use crate::contexts::member_context::{MemberDispatch, get_servers, TMemberContext};
use crate::models::server::{MultiServer, Server};
use crate::comps::modal::{Modal, toggle_modal};
use crate::views::home::HomeRoute;
use crate::views::home::me::MeRoute;
use crate::contexts::server_context::{get_server_channels, ServerDispatch, TServerContext};
use crate::utils::api_requests::{api_post, PostResponse};
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
                (String::from("name"), name.clone()),
                (String::from("server_id"), server_ctx.current_server.id.clone().to_string())
            ];
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(post_response) = api_post::<Channel>(String::from("channel"), form_data, false).await{
                    if let PostResponse::NonResponse(channel) = post_response {
                        server_ctx.dispatch(ServerDispatch::UpdateServerChannels(get_server_channels(server_ctx.current_server.id.clone()).await.unwrap()));
                        server_ctx.dispatch(ServerDispatch::UpdateChannel(channel.clone()));
                        toggle_modal("modal_overlay");
                        submit_nav.push(&ServerRoute::Channel { server_id: server_ctx.current_channel.id, channel_id: channel.id })
                    }
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