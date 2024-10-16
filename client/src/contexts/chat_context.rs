use yew::prelude::*;
use yew::html::ChildrenProps;
use std::rc::Rc;
use gloo::net::http::Request;
use serde::{Deserialize, Serialize};

use crate::models::{
    chat::{Chat, MultiChatPreview},
    member::Member,
    message::MessageThread,
    server::MultiServer
};
use crate::utils::api_requests::api_get;
use crate::utils::auth_token;

pub type TChatContext = UseReducerHandle<ChatContext>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ChatContext {
    pub previews: MultiChatPreview,
    pub current_chat: Chat,
    pub thread: MessageThread
}
pub enum ChatDispatch {
    UpdatePreviews(MultiChatPreview),
    UpdateCurrentChat(Chat),
    UpdateCurrentThread(MessageThread),
}
impl Reducible for ChatContext {
    type Action = ChatDispatch;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let new_context = match action {
            ChatDispatch::UpdatePreviews(previews) => Self {
                previews,
                current_chat: self.current_chat.clone(),
                thread: self.thread.clone()
            },
            ChatDispatch::UpdateCurrentChat(chat) => Self {
                previews: self.previews.clone(),
                current_chat: chat,
                thread: MessageThread::default()
            },
            ChatDispatch::UpdateCurrentThread(new_thread) => Self {
                previews: self.previews.clone(),
                current_chat: self.current_chat.clone(),
                thread: new_thread
            },
        };
        Rc::new(new_context)
    }
}

#[function_component(ChatProvider)]
pub fn chat_provider(props: &ChildrenProps) -> Html {
    let context = use_reducer(|| ChatContext {
        previews: MultiChatPreview::default(),
        current_chat: Chat::default(),
        thread: MessageThread::default()
    });

    {
        let context = context.clone();
        use_effect_with(context.current_chat.clone(), move |_| {
            if context.current_chat.clone().eq(&Chat::default()){
                return;
            }
            let context = context.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(thread) = get_message_thread(context.current_chat.id).await {
                    context.dispatch(ChatDispatch::UpdateCurrentThread(thread));
                }
            });
        });
    }

    html! {
        <ContextProvider<TChatContext> context={context}>
            {props.children.clone()}
        </ContextProvider<TChatContext>>
    }
}

pub async fn get_message_thread(chat_id: i32) -> Result<MessageThread, ()> {
    api_get::<MessageThread>(format!("chat/{}/thread", chat_id)).await
}