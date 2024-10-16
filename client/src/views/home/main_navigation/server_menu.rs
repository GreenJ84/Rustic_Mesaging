use yew::{function_component, Html, html};
use yew::prelude::*;
use yew_router::prelude::{Link, use_navigator};

use crate::contexts::member_context::TMemberContext;
use crate::views::{
    home::{
        HomeRoute,
        main_navigation::{
            discover_servers::DiscoverServersModal,
            new_server::NewServerForm,
            server_menu_icon::ServerMenuIcon,
        }
    }
};

#[function_component(ServerSidebar)]
pub fn server_sidebar() -> Html {
    let member_context = use_context::<TMemberContext>().unwrap();
    let nav = use_navigator().unwrap();

    html! {
        <nav id="sidebar-main">
            <ul>
                <li title="Direct Messages">
                    <Link<HomeRoute> to={HomeRoute::Me}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" fill="var(--background-medium)" class="bi bi-asterisk" viewBox="0 0 16 16">
                          <path d="M8 0a1 1 0 0 1 1 1v5.268l4.562-2.634a1 1 0 1 1 1 1.732L10 8l4.562 2.634a1 1 0 1 1-1 1.732L9 9.732V15a1 1 0 1 1-2 0V9.732l-4.562 2.634a1 1 0 1 1-1-1.732L6 8 1.438 5.366a1 1 0 0 1 1-1.732L7 6.268V1a1 1 0 0 1 1-1"/>
                        </svg>
                    </Link<HomeRoute>>
                </li>
                <hr/>
                {for member_context.servers.servers.clone().into_iter().map(|server| html!{
                    <ServerMenuIcon server={server}/>
                })}
                {if !member_context.servers.servers.is_empty()
                     { html!{ <hr/> } } else { html! {} }
                }
                <li class="new_server" title="new_server">
                    <NewServerForm />
                </li>
            </ul>
            <DiscoverServersModal />
        </nav>
    }
}

