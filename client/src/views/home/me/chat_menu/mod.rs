pub(crate) mod chat_menu_items;

use log::log;
use yew::prelude::*;
use yew_router::components::Link;
use gloo::net::http::Request;
use wasm_bindgen_futures::spawn_local;
use crate::comps::{
    icon::get_random_svg,
    member_bar::MemberBar
};
use crate::models::{
    chat::{ChatDetail, MultiChatPreview},
    message::Message
};
use crate::contexts::{
    chat_context::{ChatDispatch, TChatContext},
    member_context::TMemberContext,
};
use crate::utils::api_requests::api_get;
use crate::views::home::me::{
    chat_menu::chat_menu_items::ChatMenuItem,
    MeRoute,
    new_chat::NewChatForm,
};

#[function_component(ChatMenu)]
pub fn chat_menu() -> Html {
    let app_ctx = use_context::<TMemberContext>().unwrap();
    let chat_ctx = use_context::<TChatContext>().unwrap();

    let inner_ctx = chat_ctx.clone();
    use_effect_with((), move |_| {
        spawn_local(async move {
            if let Ok(previews) = api_get::<MultiChatPreview>(String::from("member/chats")).await {
                inner_ctx.dispatch(ChatDispatch::UpdatePreviews(previews));
            }
        });
    });
    html! {
        <div id="sidebar-sub" class="chat-menu">
            <input type={"search"} placeholder={"Find or start a conversation"}/>
            <hr/>
            <ul>
                <li title="Friends" class={"chat-menu-category"}>
                    <Link<MeRoute> to={MeRoute::Overview}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-person-raised-hand" viewBox="0 0 16 16">
                            <path d="M6 6.207v9.043a.75.75 0 0 0 1.5 0V10.5a.5.5 0 0 1 1 0v4.75a.75.75 0 0 0 1.5 0v-8.5a.25.25 0 1 1 .5 0v2.5a.75.75 0 0 0 1.5 0V6.5a3 3 0 0 0-3-3H6.236a1 1 0 0 1-.447-.106l-.33-.165A.83.83 0 0 1 5 2.488V.75a.75.75 0 0 0-1.5 0v2.083c0 .715.404 1.37 1.044 1.689L5.5 5c.32.32.5.754.5 1.207"/>
                            <path d="M8 3a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3"/>
                        </svg>
                        {"Friends"}
                    </Link<MeRoute>>
                </li>
                <li title="Nitro" class={"chat-menu-category"}>
                    <a>
                        <svg class="linkButtonIcon_c91bad" aria-hidden="true" role="img" xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="none" viewBox="0 0 24 24">
                            <path fill="currentColor" d="M15 14a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z" class=""></path>
                            <path fill="currentColor" fill-rule="evenodd" d="M7 4a1 1 0 0 0 0 2h3a1 1 0 1 1 0 2H5.5a1 1 0 0 0 0 2H8a1 1 0 1 1 0 2H6a1 1 0 1 0 0 2h1.25A8 8 0 1 0 15 4H7Zm8 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z" clip-rule="evenodd" class="">
                            </path><path fill="currentColor" d="M2.5 10a1 1 0 0 0 0-2H2a1 1 0 0 0 0 2h.5Z" class=""></path>
                        </svg>
                        {"Nitro"}
                    </a>
                </li>
                <li title="Shop" class={"chat-menu-category"}>
                    <a>
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-shop" viewBox="0 0 16 16">
                            <path d="M2.97 1.35A1 1 0 0 1 3.73 1h8.54a1 1 0 0 1 .76.35l2.609 3.044A1.5 1.5 0 0 1 16 5.37v.255a2.375 2.375 0 0 1-4.25 1.458A2.37 2.37 0 0 1 9.875 8 2.37 2.37 0 0 1 8 7.083 2.37 2.37 0 0 1 6.125 8a2.37 2.37 0 0 1-1.875-.917A2.375 2.375 0 0 1 0 5.625V5.37a1.5 1.5 0 0 1 .361-.976zm1.78 4.275a1.375 1.375 0 0 0 2.75 0 .5.5 0 0 1 1 0 1.375 1.375 0 0 0 2.75 0 .5.5 0 0 1 1 0 1.375 1.375 0 1 0 2.75 0V5.37a.5.5 0 0 0-.12-.325L12.27 2H3.73L1.12 5.045A.5.5 0 0 0 1 5.37v.255a1.375 1.375 0 0 0 2.75 0 .5.5 0 0 1 1 0M1.5 8.5A.5.5 0 0 1 2 9v6h1v-5a1 1 0 0 1 1-1h3a1 1 0 0 1 1 1v5h6V9a.5.5 0 0 1 1 0v6h.5a.5.5 0 0 1 0 1H.5a.5.5 0 0 1 0-1H1V9a.5.5 0 0 1 .5-.5M4 15h3v-5H4zm5-5a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1h-2a1 1 0 0 1-1-1zm3 0h-2v3h2z"/>
                        </svg>
                        {"Shop"}
                    </a>
                </li>
                <li class="direct-messages">
                    {"Direct Messages"}
                    <NewChatForm/>
                </li>
                {for chat_ctx.previews.chat_previews.clone().into_iter().map(|chat| html!{
                    <ChatMenuItem
                        chat={chat}
                    />
                })}
            </ul>

            <MemberBar />
        </div>
    }
}