use crate::views::home::me::MeRoute;
use crate::AppRoute;
use yew::{function_component, html, Html};
use yew_router::components::Link;

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
