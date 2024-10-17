use yew::prelude::*;
use yew_router::{components::Link, hooks::use_navigator};
use gloo::net::http::Request;
use rand::prelude::IndexedRandom;
use crate::comps::icon::{get_random_svg, Icon};
use crate::models::chat::{Chat, MultiChatPreview};
use crate::contexts::chat_context::{ChatDispatch, TChatContext};
use crate::utils::api_requests::api_delete;
use crate::views::home::me::chat_menu::chat_form::ChatForm;
use crate::views::home::me::MeRoute;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub chat: Chat
}

#[function_component(ChatMenuItem)]
pub fn chat_menu_icon(Props { chat }: &Props) -> Html {
    let chat_ctx = use_context::<TChatContext>().unwrap();
    let navigation = use_navigator().unwrap();

    let leave_chat = Callback::from({
        let inner_ctx = chat_ctx.clone();
        let chat_clone = chat.clone();
        let navigation = navigation.clone();
        move |event: MouseEvent| {
            event.stop_propagation();
            event.prevent_default();
            let chat_clone = chat_clone.clone();
            let inner_ctx = inner_ctx.clone();
            let navigation = navigation.clone();

            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(_) = api_delete(format!("chat/{}/leave", chat_clone.id)).await {
                    inner_ctx.dispatch(ChatDispatch::UpdatePreviews(
                        MultiChatPreview { chat_previews: inner_ctx.previews.chat_previews
                            .clone()
                            .into_iter()
                            .filter(|preview|
                            preview.ne(&chat_clone)
                            )
                            .collect::<Vec<Chat>>()
                        }
                    ));
                    navigation.replace(&MeRoute::Overview);
                }
            })
        }
    });

    html! {
        <li class={"sub-menu-item chat-item"}
            title={chat.name.clone()}
            onmousedown={Callback::from({
                let context = chat_ctx.clone();
                let chat_clone = chat.clone();
                move |_| {
                    context.dispatch(ChatDispatch::UpdateCurrentChat(chat_clone.clone()));
                }
            })}
        >
            <Link<MeRoute>
                to={MeRoute::ChatRoom {chat_id: chat.id}}
            >
                {get_random_svg(true, "member-icon", "40")}
                <span>{&chat.name}</span>
                <div class="item-options">
                    <ChatForm
                        entity={(chat.id.clone(), chat.name.clone())}
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
                    />
                    <button
                        class="item-delete"
                        onclick={leave_chat}
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" class="bi bi-x-lg" viewBox="0 0 16 16">
                            <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
                        </svg>
                    </button>
                </div>
            </Link<MeRoute>>
        </li>
    }
}
