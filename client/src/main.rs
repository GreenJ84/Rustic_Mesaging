#![allow(unused)]

pub(crate) mod views;
pub(crate) mod models;
pub(crate) mod contexts;
pub(crate) mod utils;
pub(crate) mod comps;

#[macro_use]
extern crate chrono;

use yew::prelude::*;
use yew_router::prelude::*;
use crate::contexts::member_context::MemberContextProvider;
use crate::views::{
    home::Home,
    landing::Landing,
    not_found::NotFound
};

#[derive(Clone, Routable, PartialEq)]
pub enum AppRoute {
    #[at("/")]
    Index,
    #[at("/app")]
    Landing,
    #[at("/app/*")]
    LandingExt,

    #[at("/servers")]
    HomeRoot,
    #[at("/servers/me")]
    Home,
    #[at("/servers/*")]
    HomeExt,

    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: AppRoute) -> Html {
    match routes {
        AppRoute::Index => html! { <Redirect<AppRoute> to={AppRoute::Landing} />},
        AppRoute::Landing | AppRoute::LandingExt => html! {
            <Landing />
        },
        AppRoute::HomeRoot => html!{ <Redirect<AppRoute> to={AppRoute::Home} />},
        AppRoute::Home | AppRoute::HomeExt => html! {
            <Home />
        },
        AppRoute::NotFound => html! { <NotFound /> },
    }
}


#[function_component(Main)]
fn app() -> Html {

    html! {
        <BrowserRouter>
            <MemberContextProvider>
                <div
                    id="modal_overlay"
                    class="closed"
                ></div>
                <Switch<AppRoute> render={switch} />
            </MemberContextProvider>
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<Main>::new().render();
}