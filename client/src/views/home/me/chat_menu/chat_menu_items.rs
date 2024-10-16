use yew::prelude::*;
use yew_router::{components::Link, hooks::use_navigator};
use gloo::net::http::Request;
use rand::prelude::IndexedRandom;
use crate::comps::icon::get_random_svg;
use crate::models::chat::{Chat, MultiChatPreview};
use crate::contexts::chat_context::{ChatDispatch, TChatContext};
use crate::views::home::me::MeRoute;
use crate::utils::auth_token;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub chat: Chat
}

#[function_component(ChatMenuItem)]
pub fn chat_menu_icon(Props { chat }: &Props) -> Html {
    let chat_ctx = use_context::<TChatContext>().unwrap();
    let navigation = use_navigator().unwrap();

    html! {
        <li class={"chat-item"}
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
                <button
                    onclick={Callback::from({
                        let inner_ctx = chat_ctx.clone();
                        let chat_clone = chat.clone();
                        move |event: MouseEvent| {
                            event.stop_propagation();
                            event.prevent_default();
                            let chat_clone = chat_clone.clone();
                            let inner_ctx = inner_ctx.clone();

                            wasm_bindgen_futures::spawn_local(async move {
                                let request = Request::delete(&format!("http://localhost:8000/chat/{}/leave", chat_clone.id))
                                    .header("Authorization", &auth_token())
                                    .send()
                                    .await;
                                match request {
                                    Ok(response) => {
                                       inner_ctx.dispatch(ChatDispatch::UpdatePreviews(
                                            MultiChatPreview { chat_previews: inner_ctx.previews.chat_previews
                                                .clone()
                                                .into_iter()
                                                .filter(|preview|
                                                    preview.ne(&chat_clone)
                                                )
                                                .collect::<Vec<Chat>>()
                                            }
                                       ))
                                    }
                                    Err(err) => log::error!("Request failed: {:?}", err),
                                }
                            })
                        }
                    })}
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-x-lg" viewBox="0 0 16 16">
                        <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
                    </svg>
                </button>
            </Link<MeRoute>>
        </li>
    }
}
