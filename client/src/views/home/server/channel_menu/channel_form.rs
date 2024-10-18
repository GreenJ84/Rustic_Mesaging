use gloo::net::http::Request;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlElement, HtmlInputElement, MouseEvent, SubmitEvent, window};
use yew::{Callback, function_component, Html, html, Properties, TargetCast, use_context, use_effect_with, use_node_ref, use_state};
use yew_router::hooks::{use_location, use_navigator};
use crate::comps::main_navigation::server_menu_item::_Props::server;
use crate::contexts::member_context::{MemberDispatch, get_servers, TMemberContext};
use crate::models::server::{MultiServer, Server};
use crate::comps::modal::{force_toggle, Modal, toggle_modal};
use crate::contexts::chat_context::{ChatDispatch, get_chat_previews};
use crate::views::home::HomeRoute;
use crate::views::home::me::MeRoute;
use crate::contexts::server_context::{get_server_channels, ServerDispatch, TServerContext};
use crate::models::channel::Channel;
use crate::models::friends::FullFriend;
use crate::utils::api_requests::{api_post, api_put, PostResponse};
use crate::views::home::server::ServerRoute;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub entity: Option<(i32, String)>,
    pub button_icon: Html,
    pub(crate) callback: Option<Callback<MouseEvent>>
}

#[function_component(ChannelForm)]
pub fn channel_form(Props { entity, button_icon, callback }: &Props) -> Html {
    let member_ctx = use_context::<TMemberContext>().unwrap();
    let server_ctx = use_context::<TServerContext>().unwrap();
    let navigation = use_navigator().unwrap();
    let entity = entity.clone();

    let new = entity.is_none();
    let name = use_state(|| if let Some(chat) = entity.clone() {
        chat.1.clone()
    } else {
        String::new()
    });
    {
        let name = name.clone();
        let entity = entity.clone();
        use_effect_with(entity.clone(), move |_|{
            name.set(if let Some(channel) = entity {
                channel.1.clone()
            } else {
                String::new()
            });
        })
    }

    let on_change = {
        let name = name.clone();
        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                name.set(input.value());
            }
        })
    };

    let on_submit = {
        let member_ctx = member_ctx.clone();
        let server_ctx = server_ctx.clone();
        let navigation = navigation.clone();
        let name = name.clone();
        let entity = entity.clone();
        let callback = callback.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let member_ctx = member_ctx.clone();
            let server_ctx = server_ctx.clone();
            let navigation = navigation.clone();
            let entity = entity.clone();
            let callback = callback.clone();

            let mut form_data = vec![
                (String::from("server_id"), server_ctx.current_server.id.clone().to_string()),
                (String::from("name"), (*name).clone()),
            ];
            if new {
                spawn_local(async move {
                    if let Ok(post_response) = api_post::<Channel>(
                        format!("channel/{}", entity.unwrap().0),
                        form_data,
                        false
                    ).await{
                        if let PostResponse::NonResponse(channel) = post_response {
                            if let Ok(channels) = get_server_channels(server_ctx.current_server.id).await {
                                server_ctx.dispatch(ServerDispatch::UpdateServerChannels(channels));
                            }
                            force_toggle("modal_overlay");
                            callback.unwrap().emit(MouseEvent::new("submit").unwrap());
                            server_ctx.dispatch(ServerDispatch::UpdateChannel(channel.clone()));
                        }
                    }
                });
            } else {
                spawn_local(async move {
                    if let Ok(channel) = api_put::<Channel>(
                        format!("channel/{}", entity.unwrap().0),
                        form_data
                    ).await{
                        if let Ok(channels) = get_server_channels(server_ctx.current_server.id).await {
                            server_ctx.dispatch(ServerDispatch::UpdateServerChannels(channels));
                        }
                        server_ctx.dispatch(ServerDispatch::UpdateChannel(channel.clone()));
                        force_toggle("modal_overlay");
                        navigation.push(&ServerRoute::Channel { server_id: server_ctx.current_server.id, channel_id: channel.id})
                    }
                });
            }
        })
    };

    let reset = {
        let entity = entity.clone();
        let name = name.clone();
        let callback = callback.clone();

        Callback::from(move |x: ()| {
            let entity = entity.clone();
            let name = name.clone();
            let callback = callback.clone();

            if let Some(chat) = entity {
                name.set(chat.1.clone());
            } else {
                name.set(String::new());
                callback.unwrap().emit(MouseEvent::new("click").unwrap())
            }
        })
    };

    html! {
        <Modal
            modal_class={"channel_form_modal"}
            button_class={"channel_form"}
            button_icon={button_icon}
            reset_state={reset}
        >
            <>
                {
                    if new {
                        html!(<h2>{"New Channel"}</h2>)
                    } else {
                        html!(<h2>{"Edit Channel"}</h2>)
                    }
                }
                <form onsubmit={on_submit} >
                    <label for="name">
                        { "Channel Name:" }
                        <input
                            type="text"
                            id="name"
                            name="name"
                            value={(*name).clone()}
                            required=true
                        />
                    </label>
                    <button type="submit" style="margin-top: 10px;">{if new { "Create" } else { "Save" } }</button>
                </form>
            </>
        </Modal>
    }
}