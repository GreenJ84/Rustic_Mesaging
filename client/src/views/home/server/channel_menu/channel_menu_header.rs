use gloo::net::http::Request;
use wasm_bindgen_futures::spawn_local;
use web_sys::MouseEvent;
use yew::{Callback, CallbackRef, function_component, Html, html, Properties, use_context, use_state};
use yew_router::hooks::use_navigator;
use crate::models::channel::NewChannel;
use crate::comps::modal::{Modal, toggle_modal};
use crate::views::home::HomeRoute;
use crate::views::home::server::channel::edit_server::EditServerForm;
use crate::contexts::member_context::{MemberDispatch, get_servers, TMemberContext};
use crate::contexts::server_context::TServerContext;
use crate::utils::api_requests::api_delete;
use crate::views::home::server::channel_menu::channel_form::ChannelForm;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub is_owner: bool
}

#[function_component(ChannelMenuHeader)]
pub fn channel_menu(Props { is_owner }: &Props) -> Html {
    let app_ctx = use_context::<TMemberContext>().unwrap();
    let server_ctx = use_context::<TServerContext>().unwrap();
    let navigation = use_navigator().unwrap();

    let options_open = use_state(|| false);
    let toggle_options = {
        let options_open = options_open.clone();
        Callback::from(move |event: MouseEvent| {
            options_open.set(!*options_open);
        })
    };

    let leave_server = {
        let app_ctx = app_ctx.clone();
        let server_ctx = server_ctx.clone();
        let nav = navigation.clone();
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            let app_ctx = app_ctx.clone();
            let server_ctx = server_ctx.clone();
            let nav = nav.clone();

            spawn_local(async move {
                if let Ok(_) = api_delete(format!("server/{}/leave", server_ctx.current_server.id)).await {
                    app_ctx.dispatch(MemberDispatch::UpdateServers(get_servers().await.unwrap()));
                    nav.push(&HomeRoute::Me);
                }
            });
        })
    };
    let delete_server = {
        let app_ctx = app_ctx.clone();
        let server_ctx = server_ctx.clone();
        let nav = navigation.clone();

        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            let app_ctx = app_ctx.clone();
            let server_ctx = server_ctx.clone();
            let nav = nav.clone();

            spawn_local(async move {
                if let Ok(_) = api_delete(format!("server/{}", server_ctx.current_server.id)).await {
                    app_ctx.dispatch(MemberDispatch::UpdateServers(get_servers().await.unwrap()));
                    nav.push(&HomeRoute::Me);
                }
            });
        })
    };

    html! {
        <div style="position: relative;">
            <div id="server-header" onclick={toggle_options.clone()}>
                {server_ctx.current_server.name.clone()}
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" class="bi bi-chevron-down" viewBox="0 0 16 16">
                {
                    if *options_open{
                        html!{
                            <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
                        }
                    } else {
                        html!{
                            <path fill-rule="evenodd" d="M1.646 4.646a.5.5 0 0 1 .708 0L8 10.293l5.646-5.647a.5.5 0 0 1 .708.708l-6 6a.5.5 0 0 1-.708 0l-6-6a.5.5 0 0 1 0-.708"/>
                        }
                    }
                }
                </svg>
            </div>
            {
                if *options_open && *is_owner {
                    html!{
                        <ul id="header_menu" style="all: unset; position: absolute; left: 50%; bottom: 0; transform: translate(-50%, calc(100% + 20px)); background-color: black; width: 90%; padding: 2%;">
                            <li>
                                <ChannelForm
                                    entity={None}
                                    button_icon={html!("Create Channel")}
                                    callback={toggle_options.clone()}
                                />
                            </li>
                            <hr/>
                            <li>
                                <EditServerForm callback={toggle_options}/>
                            </li>
                            <li onclick={delete_server}>{"Delete Server"}</li>
                            <hr/>
                            <li onclick={leave_server}>{"Leave Server"}</li>
                        </ul>
                                        }
                } else {
                    html!{}
                }
            }
        </div>

    }
}