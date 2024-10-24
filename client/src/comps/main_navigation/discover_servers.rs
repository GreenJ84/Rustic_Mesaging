use gloo::net::http::{Method, Request, RequestBuilder};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlElement, MouseEvent, SubmitEvent};
use yew::{Callback, function_component, Html, html, Properties, use_context, use_effect, use_effect_with, use_node_ref, use_state};
use yew_router::hooks::use_navigator;
use crate::contexts::member_context::{MemberDispatch, get_servers, TMemberContext};
use crate::models::server::{MultiServer, Server};
use crate::comps::modal::Modal;
use crate::models::server_membership::ServerMembership;
use crate::utils::api_requests::{api_delete, api_get, api_post};
use crate::views::home::HomeRoute;
use crate::views::home::me::MeRoute;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub(crate) switch_icon: Html
}

#[function_component(DiscoverServersModal)]
pub fn discover_servers(Props { switch_icon }: &Props) -> Html {
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
            button_icon={switch_icon}
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
                            if member.is_none() {
                                if let Ok(_) = api_post::<ServerMembership>(format!("server/{}/join",server_id), vec![], false).await {
                                    is_member.set(!*is_member);
                                    member_ctx.dispatch(MemberDispatch::UpdateServers(get_servers().await.unwrap()))
                                }
                            } else {
                                if let Ok(_) = api_delete(format!("server/{}/leave", server_id)).await {
                                    is_member.set(!*is_member);
                                    member_ctx.dispatch(MemberDispatch::UpdateServers(get_servers().await.unwrap()))
                                }
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
            <div></div>
            <span>{server.clone().icon("icon", "35")}</span>
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