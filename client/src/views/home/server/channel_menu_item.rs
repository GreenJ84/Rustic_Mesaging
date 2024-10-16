use rand::prelude::IndexedRandom;
use yew::prelude::*;
use yew::{function_component, Html, html};
use yew_router::components::Link;
use crate::models::channel::Channel;
use crate::contexts::server_context::{ServerDispatch, TServerContext};
use crate::views::home::server::ServerRoute;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub channel: Channel,
}
#[function_component(ChannelMenuItem)]
pub fn channel_menu_item(Props { channel }: &Props) -> Html{
    let server_ctx = use_context::<TServerContext>().unwrap();
    let channel = channel.clone();

    html!{
        <li
            class="channel-item"
            title={channel.name.clone()}
            onmousedown={Callback::from({
                let context = server_ctx.clone();
                let channel_clone = channel.clone();
                move |_| {
                    context.dispatch(ServerDispatch::UpdateChannel(channel_clone.clone()));
                }
            })}
        >
            <span></span>
            <Link<ServerRoute> to={ServerRoute::Channel {server_id: server_ctx.current_server.id, channel_id: channel.id}}>
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-hash" viewBox="0 0 16 16">
                    <path d="M8.39 12.648a1 1 0 0 0-.015.18c0 .305.21.508.5.508.266 0 .492-.172.555-.477l.554-2.703h1.204c.421 0 .617-.234.617-.547 0-.312-.188-.53-.617-.53h-.985l.516-2.524h1.265c.43 0 .618-.227.618-.547 0-.313-.188-.524-.618-.524h-1.046l.476-2.304a1 1 0 0 0 .016-.164.51.51 0 0 0-.516-.516.54.54 0 0 0-.539.43l-.523 2.554H7.617l.477-2.304c.008-.04.015-.118.015-.164a.51.51 0 0 0-.523-.516.54.54 0 0 0-.531.43L6.53 5.484H5.414c-.43 0-.617.22-.617.532s.187.539.617.539h.906l-.515 2.523H4.609c-.421 0-.609.219-.609.531s.188.547.61.547h.976l-.516 2.492c-.008.04-.015.125-.015.18 0 .305.21.508.5.508.265 0 .492-.172.554-.477l.555-2.703h2.242zm-1-6.109h2.266l-.515 2.563H6.859l.532-2.563z"/>
                </svg>
                {channel.name}
            </Link<ServerRoute>>
        </li>
    }
}
