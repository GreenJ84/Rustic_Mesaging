pub(crate) mod chat_room;
pub(crate) mod overview;
pub(crate) mod new_chat;
pub(crate) mod chat_menu;

use yew::{function_component, Html, html};
use yew_router::{Routable, Switch};
use yew_router::prelude::Redirect;
use crate::AppRoute;
use crate::views::home::me::{
    chat_menu::ChatMenu,
    chat_room::ChatRoom,
    overview::Overview
};

#[derive(Clone, Routable, PartialEq)]
pub enum MeRoute {
    #[at("/servers/me")]
    Overview,
    #[at("/servers/me/:chat_id")]
    ChatRoom { chat_id: i32 },
    #[not_found]
    #[at("/404")]
    NotFound,
}
fn switch(routes: MeRoute) -> Html {
    match routes {
        MeRoute::Overview => html! { <Overview />},
        MeRoute::ChatRoom { chat_id} => html! { <ChatRoom chat_id={chat_id} /> },
        MeRoute::NotFound => html!{ <Redirect<AppRoute> to={ AppRoute::NotFound }/>}
    }
}

#[function_component(Me)]
pub fn me() -> Html {
    html! {
        <>
            <ChatMenu />
            <Switch<MeRoute> render={switch} />
        </>
    }
}