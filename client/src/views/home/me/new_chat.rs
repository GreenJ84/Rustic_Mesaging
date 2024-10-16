use gloo::net::http::Request;
use web_sys::{Event, HtmlElement, HtmlInputElement, InputEvent, MouseEvent, SubmitEvent};
use yew::{Callback, function_component, Html, html, use_context, use_node_ref, use_state, use_effect_with};
use yew_router::hooks::use_navigator;
use crate::models::chat::Chat;
use crate::contexts::member_context::TMemberContext;
use crate::models::friends::FullFriend;
use crate::models::server::Server;
use crate::comps::icon::get_random_svg;
use crate::comps::modal::Modal;
use crate::views::home::HomeRoute;
use crate::contexts::chat_context::{ChatDispatch, TChatContext};
use crate::utils::api_requests::{api_post, api_put, PostResponse};
use crate::views::home::me::MeRoute;

#[function_component(NewChatForm)]
pub fn new_chat_modal() -> Html {
    let app_ctx = use_context::<TMemberContext>().unwrap();
    let chat_ctx = use_context::<TChatContext>().unwrap();
    let navigation = use_navigator().unwrap();

    let name_ref = use_node_ref();
    let selected_friends = use_state(Vec::<FullFriend>::new);
    let select_friend = {
        let selected_friends = selected_friends.clone();
        Callback::from(move |friend: FullFriend| {
            if !selected_friends.contains(&friend) {
                selected_friends.set({
                    let mut new_list = (*selected_friends).clone();
                    new_list.push(friend.clone());
                    new_list
                });
            } else {
                selected_friends.set(selected_friends.iter().filter(|&f| f != &friend).cloned().collect());
            }

        })
    };

    let on_submit = {
        let navigation = navigation.clone();
        let chat_ctx = chat_ctx.clone();
        let name_ref = name_ref.clone();
        let selected_friends = selected_friends.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let navigation = navigation.clone();
            let chat_ctx = chat_ctx.clone();

            let name = name_ref.cast::<HtmlInputElement>().unwrap().value();
            let selected_friend_ids = selected_friends.clone()
                .iter()
                .map(|friend: &FullFriend| { friend.member.id.clone() })
                .collect::<Vec<i32>>();
            let selected_friends = selected_friends.clone()
                .iter()
                .map(|friend: &FullFriend| { friend.member.username.clone() })
                .collect::<Vec<String>>();

            let mut form_data = vec![
                (String::from("name"), if name.is_empty() {selected_friends.join(",")} else {name.clone()}),
            ];
            form_data.extend(selected_friend_ids.iter().map(|id|{
                (String::from("member"), id.to_string())
            }).collect::<Vec<(String, String)>>());

            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(post_response) = api_post::<Chat>(String::from("chat"), form_data, false).await {
                    if let PostResponse::NonResponse(chat) = post_response {
                        chat_ctx.dispatch(ChatDispatch::UpdateCurrentChat(chat.clone()));
                        navigation.push(&MeRoute::ChatRoom { chat_id: chat.id });
                    }
                }
            });
        })
    };

    html! {
        <Modal
            modal_class={"new_chat_modal"}
            button_class={"new_chat"}
            button_icon={html!(
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-plus" viewBox="0 0 16 16">
                    <path d="M8 4a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 4"/>
                </svg>
            )}
        >
            <>
                <h2>{ "Start a New Chat" }</h2>
                <p>{"Add up to 9 more friends"}</p>
                <form onsubmit={on_submit} >
                    <label>
                        {"Chat Name"}
                        <input type="text" ref={name_ref} placeholder="Chatroom name"/>
                    </label>
                    <label>
                        {"Members"}
                        <div name="friends" id="friends">
                           {
                               for app_ctx.friends.friends.clone().iter().map(|friend|{
                                    let select_friend = select_friend.clone();
                                    let is_selected = selected_friends.contains(&friend);
                                   html!{
                                        <div
                                            class={if is_selected {"selected"} else {""}}
                                            key={friend.member.id.to_string()}
                                            title={friend.member.username.clone()}
                                            onclick={
                                                let friend = friend.clone();
                                                Callback::from(move |e: MouseEvent|{
                                                    e.prevent_default();
                                                    let friend = friend.clone();
                                                    select_friend.emit(friend);
                                                })
                                           }
                                        >
                                            {get_random_svg(true, "friend-item", "30")}
                                        </div>
                                   }})
                           }
                        </div>
                    </label>
                    <button type="submit" style="margin-top: 10px;">{ "Create" }</button>
                </form>
            </>
        </Modal>
    }
}