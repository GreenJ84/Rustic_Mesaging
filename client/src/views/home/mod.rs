use yew::prelude::*;
use yew_router::{BrowserRouter, Routable, Switch, components::Redirect};
use main_navigation::server_menu::ServerSidebar;
use crate::contexts::{
    member_context::TMemberContext,
    chat_context::ChatProvider,
    server_context::ServerProvider,
};
use crate::views::{
    landing::LandingRoute,
    home::{
        me::{
            Me,
        },
        server::{
            ServerComp,
        }
    },
};
use crate::views::home::profile::Profile;

pub(crate) mod me;
pub(crate) mod server;
pub(crate) mod profile;
pub(crate) mod main_navigation;
pub(crate) mod edit_member;

#[derive(Clone, Routable, PartialEq)]
pub enum HomeRoute {
    #[at("/servers/me")]
    Me,
    #[at("/servers/me/*")]
    MeExt,

    #[at("/servers/:server_id")]
    Server { server_id: i32 },
    #[at("/servers/:server_id/*")]
    ServerExt { server_id: i32 },
}

fn switch(routes: HomeRoute) -> Html {
    match routes {
        HomeRoute::Me |  HomeRoute::MeExt => html! {
            <ChatProvider>
                <Me />
            </ChatProvider>
        },
        HomeRoute::Server {server_id } |
        HomeRoute::ServerExt {server_id} => html! {
            <ServerComp server_id={server_id}/>
        }
    }
}

#[function_component(Home)]
pub fn home() -> Html {
    let member_ctx = use_context::<TMemberContext>().unwrap();
    if !member_ctx.logged_in {
        return html! { <Redirect<LandingRoute> to={LandingRoute::Login} /> };
    }
    html! {
        <div id="domain-container">
            <ServerProvider>
                <ServerSidebar />
                <Profile />
                <BrowserRouter>
                    <Switch<HomeRoute> render={switch} />
                </BrowserRouter>
            </ServerProvider>
        </div>
    }
}