use yew::{function_component, Html, html};
use yew::prelude::*;
use yew_router::prelude::{Link, use_navigator};
use crate::comps::main_navigation::{
    discover_servers::DiscoverServersModal,
    server_menu_item::ServerMenuItem,
};
use crate::comps::main_navigation::server_form::ServerForm;
use crate::contexts::member_context::TMemberContext;
use crate::contexts::server_context::{ServerDispatch, TServerContext};
use crate::models::server::Server;
use crate::views::home::HomeRoute;

#[function_component(ServerSidebar)]
pub fn server_sidebar() -> Html {
    let member_ctx = use_context::<TMemberContext>().unwrap();
    let server_ctx = use_context::<TServerContext>().unwrap();
    let nav = use_navigator().unwrap();

    html! {
        <nav id="main-menu">
            <ul>
                <li
                    title="Direct Messages"
                    onmousedown={Callback::from({
                        let server_ctx = server_ctx.clone();
                        move |_| {
                            server_ctx.dispatch(ServerDispatch::UpdateServer(Server::default()));
                        }
                    })}
                >
                    <Link<HomeRoute> to={HomeRoute::Me}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" fill="var(--background-medium)" class="bi bi-asterisk" viewBox="0 0 16 16">
                          <path d="M8 0a1 1 0 0 1 1 1v5.268l4.562-2.634a1 1 0 1 1 1 1.732L10 8l4.562 2.634a1 1 0 1 1-1 1.732L9 9.732V15a1 1 0 1 1-2 0V9.732l-4.562 2.634a1 1 0 1 1-1-1.732L6 8 1.438 5.366a1 1 0 0 1 1-1.732L7 6.268V1a1 1 0 0 1 1-1"/>
                        </svg>
                    </Link<HomeRoute>>
                </li>
                <hr/>
                {for member_ctx.servers.servers.clone().into_iter().map(|server| html!{
                    <ServerMenuItem
                        server={server.clone()}
                        selected={server_ctx.current_server.id.eq(&server.id)}
                    />
                })}
                {if !member_ctx.servers.servers.is_empty()
                     { html!{ <hr/> } } else { html! {} }
                }
                <li class="new_server" title="new_server">
                    <ServerForm
                        new={true}
                        button_icon={html!{
                            <>
                                <span></span>
                                <svg xmlns="http://www.w3.org/2000/svg" width="26" height="26" fill="currentColor" class="bi bi-patch-plus" viewBox="0 0 16 16">
                                    <path fill-rule="evenodd" d="M8 5.5a.5.5 0 0 1 .5.5v1.5H10a.5.5 0 0 1 0 1H8.5V10a.5.5 0 0 1-1 0V8.5H6a.5.5 0 0 1 0-1h1.5V6a.5.5 0 0 1 .5-.5"/>
                                    <path d="m10.273 2.513-.921-.944.715-.698.622.637.89-.011a2.89 2.89 0 0 1 2.924 2.924l-.01.89.636.622a2.89 2.89 0 0 1 0 4.134l-.637.622.011.89a2.89 2.89 0 0 1-2.924 2.924l-.89-.01-.622.636a2.89 2.89 0 0 1-4.134 0l-.622-.637-.89.011a2.89 2.89 0 0 1-2.924-2.924l.01-.89-.636-.622a2.89 2.89 0 0 1 0-4.134l.637-.622-.011-.89a2.89 2.89 0 0 1 2.924-2.924l.89.01.622-.636a2.89 2.89 0 0 1 4.134 0l-.715.698a1.89 1.89 0 0 0-2.704 0l-.92.944-1.32-.016a1.89 1.89 0 0 0-1.911 1.912l.016 1.318-.944.921a1.89 1.89 0 0 0 0 2.704l.944.92-.016 1.32a1.89 1.89 0 0 0 1.912 1.911l1.318-.016.921.944a1.89 1.89 0 0 0 2.704 0l.92-.944 1.32.016a1.89 1.89 0 0 0 1.911-1.912l-.016-1.318.944-.921a1.89 1.89 0 0 0 0-2.704l-.944-.92.016-1.32a1.89 1.89 0 0 0-1.912-1.911z"/>
                                </svg>
                            </>
                        }}
                        callback={Option::<Callback<MouseEvent>>::None}
                    />
                </li>
            </ul>
            <DiscoverServersModal
                switch_icon={html!(
                <svg fill="currentColor" enable-background="new 0 0 512 512" viewBox="0 0 512 512" width="32" height="32" xmlns="http://www.w3.org/2000/svg">
                    <g><g>
                        <path d="m255.5 226.2c-16.9 0-30 13.1-29.9 29.8.3 16.8 13.9 30.2 30.7 30.3 16.2.1 29.3-13 29.4-29.2 0-.2 0-.4 0-.6.2-16.6-13-30.2-29.6-30.4-.2.1-.4.1-.6.1z"/>
                        <path d="m256 0c-141.4 0-256 114.6-256 256s114.6 256 256 256 256-114.6 256-256-114.6-256-256-256zm135.6 144.5c-21.8 56.1-43.9 112-65.8 168-2.2 6.1-6.9 10.8-12.9 13.1-56.1 22-112.3 44.1-168.4 66.2-1.9.7-3.8 1.2-5.7 1.6-15 .1-23.4-12.8-18.3-26 11-28.5 22.3-56.9 33.5-85.3 10.8-27.4 21.5-54.7 32.2-82.2 2.5-6.5 6.6-11.1 13.1-13.6 55.8-21.8 111.6-43.7 167.4-65.7 12.1-4.8 23.3 0 26 11.6.7 4.1.4 8.4-1.1 12.3z"/>
                    </g></g>
                </svg>
            )}
            />
        </nav>
    }
}

