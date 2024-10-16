use rand::prelude::IndexedRandom;
use yew::prelude::*;
use yew::{function_component, Html, html};
use yew_router::components::Link;
use crate::comps::icon::get_random_svg;
use crate::models::server::Server;
use crate::views::home::HomeRoute;
use crate::contexts::server_context::{ServerDispatch, TServerContext};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub server: Server,
}
#[function_component(ServerMenuIcon)]
pub fn server_menu_icon(Props { server }: &Props) -> Html{
    let server_ctx = use_context::<TServerContext>().unwrap();
    let server = server.clone();

    html!{
        <li
            class={classes!("server-menu-icon", "tooltip")}
            title={server.name.clone()}
            onmousedown={Callback::from({
                let context = server_ctx.clone();
                let server_clone = server.clone();
                move |_| {
                    server_ctx.dispatch(ServerDispatch::UpdateServer(server_clone.clone()));
                }
            })}
        >
            <Link<HomeRoute> to={HomeRoute::Server {server_id: server.id}}>
                <span></span>
                {
                    html!{ get_random_svg(false, "server-icon", "20") }
                }
            </Link<HomeRoute>>
        </li>
    }
}
