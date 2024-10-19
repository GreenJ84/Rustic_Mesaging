mod message_item;

use std::fmt::Debug;
use std::ops::Deref;
use gloo::net::http::Request;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlTextAreaElement, InputEvent, KeyboardEvent, SubmitEvent};
use yew::{Callback, function_component, Html, html, Properties, use_context, use_effect_with, use_node_ref, use_state};
use crate::contexts::member_context::{MemberDispatch, get_servers, TMemberContext};
use crate::models::message::{Message, MessageThread};
use crate::models::server::{MultiServer, Server};
use crate::comps::icon::get_random_svg;
use crate::comps::modal::toggle_modal;
use crate::views::home::HomeRoute;
use crate::contexts::chat_context::{ChatContext, ChatDispatch, get_message_thread, TChatContext};
use crate::utils::api_requests::api_post;
use crate::views::home::me::chat_room::message_item::MessageItem;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub chat_id: i32
}
#[function_component(ChatRoom)]
pub fn direct_message(Props { chat_id }: &Props) -> Html {
    let member_ctx = use_context::<TMemberContext>().unwrap();
    let chat_ctx = use_context::<TChatContext>().unwrap();

    let chat_ref = use_node_ref();
    {
        let chat_ref = chat_ref.clone();
        let messages = chat_ctx.thread.thread.clone();
        use_effect_with(messages, move |_| {
            if let Some(channel_el) = chat_ref.cast::<web_sys::Element>() {
                channel_el.set_scroll_top(channel_el.scroll_height());
            }
            || ()
        });
    }

    let textarea_ref = use_node_ref();
    let on_submit = {
        let textarea_ref = textarea_ref.clone();
        let member_ctx = member_ctx.clone();
        let chat_ctx = chat_ctx.clone();

        Callback::from(move |e: SubmitEvent| {
            let member_ctx = member_ctx.clone();
            let chat_ctx = chat_ctx.clone();
            let message: HtmlTextAreaElement = textarea_ref.cast::<HtmlTextAreaElement>().unwrap();
            let mut form_data = vec![
                (String::from("content"), message.value().clone()),
                (String::from("sender_id"), member_ctx.member.id.to_string()),
                (String::from("chat_id"), chat_ctx.current_chat.id.to_string()),
            ];
            spawn_local(async move {
                if let Ok(post_response) = api_post::<Message>(String::from("message"), form_data, false).await {
                    message.set_value("");
                    if let Ok(thread) = get_message_thread(chat_ctx.current_chat.id).await {
                        chat_ctx.dispatch(ChatDispatch::UpdateCurrentThread(thread.clone()));
                    }
                }
            });
        })
    };

    html! {
        <main id="main-content" class="chat-room">
            <div id="chat_room-header" class="header">
                {get_random_svg(true, "user-icon", "20")}
                <span>{&chat_ctx.current_chat.name}</span>
            </div>
            <hr/>
            <section id="message-thread" class="thread" ref={chat_ref}>
                {get_random_svg(true, "user-icon", "20")}
                <h1>{&chat_ctx.current_chat.name}</h1>
                <h2>{chat_ctx.current_chat.created_at.format("%Y-%m-%d %H:%M:%S").to_string()}</h2>
                <p>{format!("This is the beginning of the message history for {}", &chat_ctx.current_chat.name)}</p>
                <hr/>
                {
                    for chat_ctx.thread.thread.clone().into_iter().rev().map(|message| html!{
                        <MessageItem
                            message={message.clone()}
                        />
                    })
                }
            </section>
            <div id="message-input" class="input">
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-plus-circle-fill" viewBox="0 0 16 16">
                    <path d="M16 8A8 8 0 1 1 0 8a8 8 0 0 1 16 0M8.5 4.5a.5.5 0 0 0-1 0v3h-3a.5.5 0 0 0 0 1h3v3a.5.5 0 0 0 1 0v-3h3a.5.5 0 0 0 0-1h-3z"/>
                </svg>
                <textarea
                    name="message"
                    placeholder={format!("Message @{}", &chat_ctx.current_chat.name)}
                    wrap="hard"
                    rows={2}
                    cols={60}
                    ref={textarea_ref.clone()}
                    oninput={Callback::from(|e: InputEvent| {
                        e.prevent_default();
                        let target = e.target().unwrap();
                        let textarea: HtmlTextAreaElement = target.dyn_into::<HtmlTextAreaElement>().unwrap();

                        textarea.style().set_property("height", "auto").unwrap();

                        let scroll_height = textarea.scroll_height();
                        textarea.style().set_property("height", &format!("{}px", scroll_height)).unwrap();
                    })}
                    onkeydown={Callback::from(move |e: KeyboardEvent| {
                        if e.key() == "Enter" && !e.shift_key() {
                            e.prevent_default();
                            on_submit.emit(SubmitEvent::new("submit").unwrap());
                        }
                    })}
                >
                </textarea>
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-gift-fill" viewBox="0 0 16 16">
                    <path d="M3 2.5a2.5 2.5 0 0 1 5 0 2.5 2.5 0 0 1 5 0v.006c0 .07 0 .27-.038.494H15a1 1 0 0 1 1 1v1a1 1 0 0 1-1 1H1a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h2.038A3 3 0 0 1 3 2.506zm1.068.5H7v-.5a1.5 1.5 0 1 0-3 0c0 .085.002.274.045.43zM9 3h2.932l.023-.07c.043-.156.045-.345.045-.43a1.5 1.5 0 0 0-3 0zm6 4v7.5a1.5 1.5 0 0 1-1.5 1.5H9V7zM2.5 16A1.5 1.5 0 0 1 1 14.5V7h6v9z"/>
                </svg>
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-emoji-smile-fill" viewBox="0 0 16 16">
                    <path d="M8 16A8 8 0 1 0 8 0a8 8 0 0 0 0 16M7 6.5C7 7.328 6.552 8 6 8s-1-.672-1-1.5S5.448 5 6 5s1 .672 1 1.5M4.285 9.567a.5.5 0 0 1 .683.183A3.5 3.5 0 0 0 8 11.5a3.5 3.5 0 0 0 3.032-1.75.5.5 0 1 1 .866.5A4.5 4.5 0 0 1 8 12.5a4.5 4.5 0 0 1-3.898-2.25.5.5 0 0 1 .183-.683M10 8c-.552 0-1-.672-1-1.5S9.448 5 10 5s1 .672 1 1.5S10.552 8 10 8"/>
                </svg>
            </div>
        </main>
    }
}
