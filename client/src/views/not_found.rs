use yew::{function_component, Html, html};
use yew_router::components::Link;
use crate::AppRoute;
use crate::views::home::me::MeRoute;

#[function_component(NotFound)]
pub fn not_found() -> Html {
    html! {
        <main>
            <div class="not-found-container" style="text-align: center; padding: 50px;">
                <h1>{ "404 - Page Not Found" }</h1>
                <p>{ "The page you are looking for doesn't exist." }</p>
            <Link<MeRoute> to={MeRoute::Overview}>{ "Go back home" }</Link<MeRoute>>
            </div>
        </main>
    }
}