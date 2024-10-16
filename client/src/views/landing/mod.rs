pub(crate) mod register;
pub(crate) mod login;

use yew::prelude::*;
use yew_router::{Routable, Switch};
use yew_router::components::{Link, Redirect};

use crate::contexts::member_context::TMemberContext;
use crate::views::{
    home::me::MeRoute,
    landing::{
        login::Login,
        register::Register
    }
};

#[derive(Clone, Routable, PartialEq)]
pub enum LandingRoute {
    #[at("/app/login")]
    Login,
    #[at("/app/register")]
    Register,
}
fn switch(routes: LandingRoute) -> Html {
    match routes {
        LandingRoute::Login => html! { <Login /> },
        LandingRoute::Register => html! { <Register /> },
    }
}

#[function_component(Landing)]
pub fn landing() -> Html {
    let member_ctx = use_context::<TMemberContext>().unwrap();
    if member_ctx.logged_in {
        return html! { <Redirect<MeRoute> to={MeRoute::Overview} /> };
    }
    html! {
        <main id="landing-page">
            <header>
                <h1>{ "Real-Time Chat" }</h1>
                <p>{ "Connect instantly. Anywhere, anytime." }</p>
            </header>

            <section>
                <p>
                    { "Welcome to Real-Time Chat, the platform where instant messaging meets reliability. Join channels, connect with your friends, and enjoy secure, fast, and real-time communication." }
                </p>
            </section>

            <section>
                <div>
                    <Link<LandingRoute> to={LandingRoute::Register}>
                        { "Sign Up" }
                    </Link<LandingRoute>>
                    <Link<LandingRoute> to={LandingRoute::Login}>
                        { "Login" }
                    </Link<LandingRoute>>
                </div>
            </section>

            <footer>
                <p>{ "© 2024 Real-Time Chat. All rights reserved." }</p>
            </footer>
            <Switch<LandingRoute> render={switch} />
        </main>
    }
}
