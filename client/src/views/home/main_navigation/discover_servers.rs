use gloo::net::http::{Method, Request, RequestBuilder};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlElement, MouseEvent, SubmitEvent};
use yew::{Callback, function_component, Html, html, Properties, use_context, use_effect, use_effect_with, use_node_ref, use_state};
use yew_router::hooks::use_navigator;
use crate::contexts::member_context::{MemberDispatch, get_servers, TMemberContext};
use crate::models::server::{MultiServer, Server};
use crate::utils::auth_token;
use crate::comps::modal::Modal;
use crate::models::server_membership::ServerMembership;
use crate::utils::api_requests::{api_delete, api_get, api_post};
use crate::views::home::HomeRoute;
use crate::views::home::me::MeRoute;

#[function_component(DiscoverServersModal)]
pub fn discover_servers() -> Html {
    let member_ctx = use_context::<TMemberContext>().unwrap();
    let servers_state = use_state(|| MultiServer::default());

    {
        let servers_state = servers_state.clone();
        use_effect_with(member_ctx.servers.clone(), move|_|{
            spawn_local(async move {
                let servers = api_get::<MultiServer>(String::from("server/all")).await.unwrap();
                servers_state.set(servers);
            })
        });
    }

    html! {
        <Modal
            modal_class={"discover_servers_modal"}
            button_class={"discover_servers"}
            button_icon={html!(
                <svg fill="currentColor" enable-background="new 0 0 512 512" viewBox="0 0 512 512" width="32" height="32" xmlns="http://www.w3.org/2000/svg">
                    <g><g>
                        <path d="m255.5 226.2c-16.9 0-30 13.1-29.9 29.8.3 16.8 13.9 30.2 30.7 30.3 16.2.1 29.3-13 29.4-29.2 0-.2 0-.4 0-.6.2-16.6-13-30.2-29.6-30.4-.2.1-.4.1-.6.1z"/>
                        <path d="m256 0c-141.4 0-256 114.6-256 256s114.6 256 256 256 256-114.6 256-256-114.6-256-256-256zm135.6 144.5c-21.8 56.1-43.9 112-65.8 168-2.2 6.1-6.9 10.8-12.9 13.1-56.1 22-112.3 44.1-168.4 66.2-1.9.7-3.8 1.2-5.7 1.6-15 .1-23.4-12.8-18.3-26 11-28.5 22.3-56.9 33.5-85.3 10.8-27.4 21.5-54.7 32.2-82.2 2.5-6.5 6.6-11.1 13.1-13.6 55.8-21.8 111.6-43.7 167.4-65.7 12.1-4.8 23.3 0 26 11.6.7 4.1.4 8.4-1.1 12.3z"/>
                    </g></g>
                </svg>
            )}
        >
            <>
                <h2>{ "Discover Servers" }</h2>
                <ul>
                    { for servers_state.servers.clone().into_iter().map(|server| {
                        html!(
                            <ServerDiscoveryItem server={server}/>
                        )
                    })}
                </ul>
            </>
        </Modal>
    }
}

#[derive(Properties, PartialEq)]
pub struct ServerItemProps{
    server: Server,
}
#[function_component(ServerDiscoveryItem)]
fn discover_server_item(ServerItemProps { server }: &ServerItemProps) -> Html {
    let member_ctx: TMemberContext = use_context::<TMemberContext>().unwrap();
    let is_member = use_state(|| false);
    let navigation = use_navigator().unwrap();

    {
        let member_ctx: TMemberContext = member_ctx.clone();
        let is_member = is_member.clone();
        let server = server.clone();
        use_effect_with(member_ctx.servers.clone(), move |_| {
            is_member.set(member_ctx.servers.servers.contains(&server));
        });
    }

    let on_click = {
        let is_member = is_member.clone();
        let member_ctx = member_ctx.clone();
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            let is_member = is_member.clone();
            let member_ctx = member_ctx.clone();

            if let Some(target) = event.target() {
                if let Some(element) = target.dyn_into::<HtmlElement>().ok() {
                    if let Some(server_id) = element.get_attribute("data-server-id") {
                        let member = element.get_attribute("member");
                        spawn_local(async move {
                            let request = RequestBuilder::new(
                                &format!("http://localhost:8000/server/{}/{}",
                                         server_id,
                                         if member.is_none() { "join" } else { "leave" }
                                ))
                                .method(if member.is_none() {Method::POST} else {Method::DELETE})
                                .header("Content-Type", "application/x-www-form-urlencoded")
                                .header("Authorization", &auth_token())
                                .build()
                                .unwrap()
                                .send()
                                .await;
                            match request {
                                Ok(response) => {
                                    // Check if the response is okay (2xx status)
                                    if response.ok() {
                                        is_member.set(!*is_member);
                                        member_ctx.dispatch(MemberDispatch::UpdateServers(get_servers().await.unwrap()))
                                    } else { log::error!("Request failed with status: {}", response.status()); }
                                }
                                // Handle request failure
                                Err(err) => { log::error!("Failed to send request: {:?}", err); }
                            }
                        });
                    } else {
                        log::error!("No server_id found on the clicked element.");
                    }
                } else {
                    log::error!("Event target is not an HtmlElement.");
                }
            }
        })
    };

    html!{
        <li>
            <h3>{server.name.clone()}</h3>
            <p>{server.description.clone()}</p>
            {if *is_member{
               html!(<button
                        onclick={on_click.clone()}
                        member={"true"}
                        class={"member"}
                        data-server-id={server.id.to_string()}
                    >{ "Leave" }</button>)
            } else {
                html!(<button
                        onclick={on_click.clone()}
                        data-server-id={server.id.to_string()}
                    >{ "Join" }</button>)
            }}
        </li>
    }
}