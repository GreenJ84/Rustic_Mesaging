use web_sys::MouseEvent;
use yew::{Callback, function_component, Html, html, use_context};
use yew_router::prelude::Link;
use crate::contexts::member_context::TMemberContext;
use crate::models::channel::Channel;
use crate::comps::member_bar::MemberBar;
use crate::contexts::server_context::{ServerDispatch, TServerContext};
use crate::views::home::server::channel_menu::channel_form::ChannelForm;
use crate::views::home::server::channel_menu::channel_menu_item::ChannelMenuItem;
use crate::views::home::server::channel_menu::channel_menu_header::ChannelMenuHeader;
use crate::views::home::server::ServerRoute;

pub(crate) mod channel_menu_header;
pub(crate) mod channel_menu_item;
pub(crate) mod channel_form;
mod server_membership_report;

#[function_component(ChannelMenu)]
pub fn channel_menu() -> Html {
    let app_ctx = use_context::<TMemberContext>().unwrap();
    let server_ctx = use_context::<TServerContext>().unwrap();

    let is_owner = app_ctx.member.id.clone().eq(&server_ctx.current_server.owner_id.clone());
    html! {
        <div id="sidebar-sub" class="channels">
            <ChannelMenuHeader is_owner={is_owner} />
            <hr/>

            <p id={"browse-channel"} class={"disabled"}>
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" class="bi bi-binoculars-fill" viewBox="0 0 16 16">
                    <path d="M4.5 1A1.5 1.5 0 0 0 3 2.5V3h4v-.5A1.5 1.5 0 0 0 5.5 1zM7 4v1h2V4h4v.882a.5.5 0 0 0 .276.447l.895.447A1.5 1.5 0 0 1 15 7.118V13H9v-1.5a.5.5 0 0 1 .146-.354l.854-.853V9.5a.5.5 0 0 0-.5-.5h-3a.5.5 0 0 0-.5.5v.793l.854.853A.5.5 0 0 1 7 11.5V13H1V7.118a1.5 1.5 0 0 1 .83-1.342l.894-.447A.5.5 0 0 0 3 4.882V4zM1 14v.5A1.5 1.5 0 0 0 2.5 16h3A1.5 1.5 0 0 0 7 14.5V14zm8 0v.5a1.5 1.5 0 0 0 1.5 1.5h3a1.5 1.5 0 0 0 1.5-1.5V14zm4-11H9v-.5A1.5 1.5 0 0 1 10.5 1h1A1.5 1.5 0 0 1 13 2.5z"/>
                </svg>
                {"Browse Channels"}
            </p>
            if is_owner {
                 <ChannelForm
                    entity={None}
                    button_icon={html!(<>
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" class="bi bi-patch-plus" viewBox="0 0 16 16">
                            <path fill-rule="evenodd" d="M8 5.5a.5.5 0 0 1 .5.5v1.5H10a.5.5 0 0 1 0 1H8.5V10a.5.5 0 0 1-1 0V8.5H6a.5.5 0 0 1 0-1h1.5V6a.5.5 0 0 1 .5-.5"/>
                            <path d="m10.273 2.513-.921-.944.715-.698.622.637.89-.011a2.89 2.89 0 0 1 2.924 2.924l-.01.89.636.622a2.89 2.89 0 0 1 0 4.134l-.637.622.011.89a2.89 2.89 0 0 1-2.924 2.924l-.89-.01-.622.636a2.89 2.89 0 0 1-4.134 0l-.622-.637-.89.011a2.89 2.89 0 0 1-2.924-2.924l.01-.89-.636-.622a2.89 2.89 0 0 1 0-4.134l.637-.622-.011-.89a2.89 2.89 0 0 1 2.924-2.924l.89.01.622-.636a2.89 2.89 0 0 1 4.134 0l-.715.698a1.89 1.89 0 0 0-2.704 0l-.92.944-1.32-.016a1.89 1.89 0 0 0-1.911 1.912l.016 1.318-.944.921a1.89 1.89 0 0 0 0 2.704l.944.92-.016 1.32a1.89 1.89 0 0 0 1.912 1.911l1.318-.016.921.944a1.89 1.89 0 0 0 2.704 0l.92-.944 1.32.016a1.89 1.89 0 0 0 1.911-1.912l-.016-1.318.944-.921a1.89 1.89 0 0 0 0-2.704l-.944-.92.016-1.32a1.89 1.89 0 0 0-1.912-1.911z"/>
                        </svg>
                        {"Add new Channel"}
                    </>)}
                    callback={Option::<Callback<MouseEvent>>::None}
                 />
            }
            <hr/>

            <ul class={if is_owner {"owner"} else {""}}>
                <li
                    class={format!("sub-menu-item channel-item{}",
                        if server_ctx.current_channel.eq(&Channel::default()) {" active"} else {""})
                    }
                    title={"Welcome"}
                    onmousedown={Callback::from({
                        let context = server_ctx.clone();
                        move |_| {
                            context.dispatch(ServerDispatch::UpdateChannel(Channel::default()));
                        }
                    })}
                >
                    <span></span>
                    <Link<ServerRoute> to={ServerRoute::Welcome {server_id: server_ctx.current_server.id}}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-hash" viewBox="0 0 16 16">
                            <path d="M8.39 12.648a1 1 0 0 0-.015.18c0 .305.21.508.5.508.266 0 .492-.172.555-.477l.554-2.703h1.204c.421 0 .617-.234.617-.547 0-.312-.188-.53-.617-.53h-.985l.516-2.524h1.265c.43 0 .618-.227.618-.547 0-.313-.188-.524-.618-.524h-1.046l.476-2.304a1 1 0 0 0 .016-.164.51.51 0 0 0-.516-.516.54.54 0 0 0-.539.43l-.523 2.554H7.617l.477-2.304c.008-.04.015-.118.015-.164a.51.51 0 0 0-.523-.516.54.54 0 0 0-.531.43L6.53 5.484H5.414c-.43 0-.617.22-.617.532s.187.539.617.539h.906l-.515 2.523H4.609c-.421 0-.609.219-.609.531s.188.547.61.547h.976l-.516 2.492c-.008.04-.015.125-.015.18 0 .305.21.508.5.508.265 0 .492-.172.554-.477l.555-2.703h2.242zm-1-6.109h2.266l-.515 2.563H6.859l.532-2.563z"/>
                        </svg>
                        <span>{"Welcome"}</span>
                    </Link<ServerRoute>>
                </li>
                {for server_ctx.channels.channels.clone().into_iter().map(|channel: Channel| html!{
                   <ChannelMenuItem channel={channel.clone()} is_owner={is_owner} />
                })}
            </ul>

            <MemberBar />
        </div>
    }
}