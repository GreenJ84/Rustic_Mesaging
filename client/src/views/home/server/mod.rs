use yew::{function_component, Html, html, Properties};
use yew_router::{Routable, Switch};
use crate::views::home::server::channel::ChannelComp;
use crate::views::home::server::channel_menu::ChannelMenu;
use channel::welcome::ServerWelcome;

pub(crate) mod channel_menu;
pub(crate) mod channel;
pub(crate) mod channel_menu_item;
mod channel_menu_header;

#[derive(Routable, Clone, PartialEq)]
pub enum ServerRoute {
    #[at("/servers/:server_id/:channel_id")]
    Channel { server_id: i32, channel_id: i32 },
    #[at("/servers/:server_id")]
    Welcome { server_id: i32 }
}
fn switch(routes: ServerRoute) -> Html {
    match routes {
        ServerRoute::Channel { channel_id: i32, ..} => html! {
            <ChannelComp />
        },
        ServerRoute::Welcome { .. }=> html! { <ServerWelcome />},
    }
}

#[derive(Properties, PartialEq, Debug)]
pub struct Props{
    pub server_id: i32
}
#[function_component(ServerComp)]
pub fn server(Props { server_id }: &Props) -> Html {

    html! {
        <>
            <ChannelMenu/>
            <Switch<ServerRoute> render={switch} />
        </>
    }
}