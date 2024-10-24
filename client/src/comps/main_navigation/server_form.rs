use gloo::net::http::Request;
use rand::seq::IndexedRandom;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlElement, HtmlInputElement, HtmlTextAreaElement, MouseEvent, SubmitEvent, window};
use yew::{Callback, function_component, Html, html, Properties, TargetCast, use_context, use_node_ref, use_state};
use yew_router::hooks::use_navigator;
use crate::comps::icon::{random_color, server_icons};
use crate::contexts::member_context::{MemberDispatch, get_servers, TMemberContext};
use crate::models::server::{MultiServer, Server};
use crate::comps::modal::{force_toggle, Modal, toggle_modal};
use crate::contexts::server_context::{ServerDispatch, TServerContext};
use crate::models::friends::FullFriend;
use crate::utils::api_requests::{api_post, api_put, PostResponse};
use crate::views::home::HomeRoute;
use crate::views::home::me::MeRoute;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub(crate) new: bool,
    pub(crate) button_icon: Html,
    pub(crate) callback: Option<Callback<MouseEvent>>
}

#[function_component(ServerForm)]
pub fn server_form(Props {new, button_icon, callback}: &Props) -> Html {
    let member_ctx = use_context::<TMemberContext>().unwrap();
    let server_ctx = use_context::<TServerContext>().unwrap();
    let navigation = use_navigator().unwrap();
    let new = new.clone();

    let name = use_state(|| if new {
        String::new()
    } else {
        server_ctx.current_server.name.clone()
    });
    let description = use_state(|| if new {
        String::new()
    } else {
        server_ctx.current_server.description.clone()
    });

    let on_name_change = {
        let name = name.clone();
        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                name.set(input.value());
            }
        })
    };
    let on_description_change = {
        let description = description.clone();
        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<HtmlTextAreaElement>() {
                description.set(input.value());
            }
        })
    };

    let on_submit = {
        let server_ctx = server_ctx.clone();
        let name = name.clone();
        let description = description.clone();
        let submit_nav = navigation.clone();
        let callback = callback.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let member_ctx = member_ctx.clone();
            let server_ctx = server_ctx.clone();
            let submit_nav = submit_nav.clone();
            let name = name.clone();
            let description = description.clone();
            let callback = callback.clone();

            let icon = server_icons("", "", "")
                .keys()
                .map(|i| i.to_owned())
                .collect::<Vec<String>>()
                .choose(&mut rand::thread_rng())
                .unwrap_or(&format!("default:{}", random_color()))
                .to_owned();

            let mut form_data = vec![
                ("name".to_string(), (*name).clone()),
                ("description".to_string(), (*description).clone()),
                ("icon".to_string(), icon.to_owned()),
                ("owner_id".to_string(), member_ctx.member.id.clone().to_string())
            ];
            if new {
                spawn_local(async move {
                    if let Ok(request) = api_post::<Server>(String::from("server"), form_data, false).await {
                        if let PostResponse::NonResponse(server) = request {
                            member_ctx.dispatch(MemberDispatch::UpdateServers(get_servers().await.unwrap()));
                            server_ctx.dispatch(ServerDispatch::UpdateServer(server.clone()));
                            force_toggle("modal_overlay");
                            if let Some(func) = callback {
                                func.emit(MouseEvent::new("click").unwrap());
                            }
                            submit_nav.push(&HomeRoute::Server { server_id: server.id });
                        }
                    }
                })
            } else {
                spawn_local(async move {
                    if let Ok(server) = api_put::<Server>(format!("server/{}", server_ctx.current_server.id), form_data).await {
                        member_ctx.dispatch(MemberDispatch::UpdateServers(get_servers().await.unwrap()));
                        server_ctx.dispatch(ServerDispatch::UpdateServer(server.clone()));
                        force_toggle("modal_overlay");
                        if let Some(func) = callback {
                            func.emit(MouseEvent::new("click").unwrap());
                        }
                        submit_nav.push(&HomeRoute::Server { server_id: server.id });
                    }
                })
            }
        })
    };

    let reset = {
        let server_ctx = server_ctx.clone();
        let name = name.clone();
        let description = description.clone();
        Callback::from(move |x: ()| {
            let server_ctx = server_ctx.clone();

            if new {
                name.set(String::new());
                description.set(String::new());
            } else {
                name.set(server_ctx.current_server.name.clone());
                description.set(server_ctx.current_server.description.clone());
            }
        })
    };

    html! {
        <Modal
            modal_class={"server_form_modal"}
            button_class={"server_form"}
            button_icon={button_icon}
            reset_state={reset}
        >
            <>
                {
                    if new {
                        html!{<>
                            <h2>{ "Start a New Server" }</h2>
                        </>}
                    } else {
                        html!{
                            <h2>{ "Edit Server" }</h2>
                        }
                    }
                }
                <form onsubmit={on_submit.clone()} >
                    <label for="name">
                        { "Server Name:" }
                        <input
                            value={(*name).clone()}
                            type="text"
                            id="name"
                            name="name"
                            onchange={on_name_change}
                            required=true
                        />
                    </label>
                    <br/>
                    <label for="description">
                        { "Description:" }
                        <textarea
                            value={(*description).clone()}
                            id="description"
                            name="description"
                            onchange={on_description_change}
                            required=true
                        ></textarea>
                    </label>
                    <br/>
                    <button type="submit" style="margin-top: 10px;">{if new { "Create" } else { "Save" }}</button>
                </form>
            </>
        </Modal>
    }
}