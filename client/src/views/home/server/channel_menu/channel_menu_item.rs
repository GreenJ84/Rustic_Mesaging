use rand::prelude::IndexedRandom;
use yew::prelude::*;
use yew::{function_component, Html, html};
use yew_router::components::Link;
use crate::comps::icon::Icon;
use crate::contexts::chat_context::ChatDispatch;
use crate::models::channel::{Channel, MultiChannel};
use crate::contexts::server_context::{ServerDispatch, TServerContext};
use crate::models::chat::{Chat, MultiChatPreview};
use crate::models::server::MultiServer;
use crate::utils::api_requests::api_delete;
use crate::views::home::server::channel_menu::channel_form::ChannelForm;
use crate::views::home::server::ServerRoute;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub channel: Channel,
    pub is_owner: bool
}
#[function_component(ChannelMenuItem)]
pub fn channel_menu_item(Props { channel, is_owner }: &Props) -> Html{
    let server_ctx = use_context::<TServerContext>().unwrap();
    let channel = channel.clone();

    let delete_channel = Callback::from({
        let inner_ctx = server_ctx.clone();
        let channel = channel.clone();
        move |event: MouseEvent| {
            event.stop_propagation();
            event.prevent_default();
            let inner_ctx = inner_ctx.clone();
            let channel = channel.clone();

            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(_) = api_delete(format!("channel/{}", channel.id)).await {
                    inner_ctx.dispatch(ServerDispatch::UpdateServerChannels(
                        MultiChannel { channels: inner_ctx.channels.channels
                            .clone()
                            .into_iter()
                            .filter(|_channel|
                            _channel.ne(&channel)
                            )
                            .collect::<Vec<Channel>>()
                        }
                    ))
                }
            })
        }
    });

    html!{
        <li
            class={format!("sub-menu-item channel-item{}", if *is_owner {" owner"} else {""})}
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
                <span>{channel.name.clone()}</span>
                {if *is_owner {html!{
                    <div class="item-options">
                        <ChannelForm
                            entity={(channel.id.clone(), channel.name.clone())}
                            button_icon={html!(
                                <Icon
                                    class_name="icon"
                                    size="24"
                                    avatar={ html!{
                                        <>
                                            <g id="Layer_13" data-name="Layer 13" stroke="transparent" fill="currentColor">
                                                <path d="m16 30a14 14 0 1 1 14-14 14 14 0 0 1 -14 14zm0-26a12 12 0 1 0 12 12 12 12 0 0 0 -12-12zm0 17.05a2 2 0 1 0 2 2 2 2 0 0 0 -2-2zm0-7a2 2 0 1 0 2 2 2 2 0 0 0 -2-1.95zm0-7.05a2 2 0 1 0 2 2 2 2 0 0 0 -2-2z"/>
                                            </g>
                                        </>
                                    }}
                                    color={"black"}
                                    ratio={"0 0 32 32"}
                                />
                            )}
                            callback={Option::<Callback<MouseEvent>>::None}
                        />
                        <button
                            class="item-delete"
                            onclick={delete_channel}
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-x-lg" viewBox="0 0 16 16">
                                <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
                            </svg>
                        </button>
                    </div>
                }} else {html!{}}}
            </Link<ServerRoute>>
        </li>
    }
}
