use gloo::net::http::Request;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, HtmlInputElement, InputEvent, KeyboardEvent, MouseEvent, SubmitEvent, window};
use yew::{Callback, function_component, Html, html, use_context, use_node_ref, use_state, use_effect_with, Properties, TargetCast};
use yew::platform::spawn_local;
use yew_router::hooks::use_navigator;
use crate::models::chat::Chat;
use crate::contexts::member_context::TMemberContext;
use crate::models::friends::FullFriend;
use crate::models::server::Server;
use crate::comps::modal::{force_toggle, Modal};
use crate::views::home::HomeRoute;
use crate::contexts::chat_context::{ChatDispatch, get_chat_previews, TChatContext};
use crate::utils::api_requests::{api_post, api_put, PostResponse};
use crate::views::home::me::MeRoute;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub entity: Option<(i32, String)>,
    pub button_icon: Html
}

#[function_component(ChatForm)]
pub fn chat_form(Props { entity, button_icon }: &Props) -> Html {
    let member_ctx = use_context::<TMemberContext>().unwrap();
    let chat_ctx = use_context::<TChatContext>().unwrap();
    let navigation = use_navigator().unwrap();
    let entity = entity.clone();



    let new = entity.is_none();
    let name = use_state(|| if let Some(chat) = entity.clone() {
            chat.1.clone()
        } else {
            String::new()
    });
    {
        let name = name.clone();
        let entity = entity.clone();
        use_effect_with(entity.clone(), move |_|{
            name.set(if let Some(chat) = entity {
                chat.1.clone()
            } else {
                String::new()
            });
        })
    }

    let on_change = {
        let name = name.clone();
        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                name.set(input.value());
            }
        })
    };

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
        let name = name.clone();
        let selected_friends = selected_friends.clone();
        let entity = entity.clone();


        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let navigation = navigation.clone();
            let chat_ctx = chat_ctx.clone();
            let selected_friends = selected_friends.clone();
            let name = name.clone();
            let entity = entity.clone();


            let selected_friends_names = selected_friends.clone()
                .iter()
                .map(|friend| { friend.member.username.clone() })
                .collect::<Vec<String>>();
            let mut form_data = vec![
                (String::from("name"), if name.clone().is_empty() {selected_friends_names.join(",")} else { name.clone().to_string()}),
            ];

            if new {
                let selected_friend_ids = selected_friends.clone()
                    .iter()
                    .map(|friend: &FullFriend| { friend.member.id.clone() })
                    .collect::<Vec<i32>>();

                form_data.extend(selected_friend_ids.iter().map(|id| {
                    (String::from("members"), id.to_string())
                }).collect::<Vec<(String, String)>>());

                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(post_response) = api_post::<Chat>(String::from("chat"), form_data, false).await {
                        if let PostResponse::NonResponse(chat) = post_response {
                            if let Ok(previews) = get_chat_previews().await {
                                chat_ctx.dispatch(ChatDispatch::UpdatePreviews(previews));
                            }
                            chat_ctx.dispatch(ChatDispatch::UpdateCurrentChat(chat.clone()));
                            navigation.push(&MeRoute::ChatRoom { chat_id: chat.id });
                            force_toggle("modal_overlay");
                        }
                    }
                });
                return;
            }

            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(chat) = api_put::<Chat>(format!("chat/{}", entity.clone().unwrap().0), form_data).await {
                    if let Ok(previews) = get_chat_previews().await {
                        chat_ctx.dispatch(ChatDispatch::UpdatePreviews(previews));
                    }
                    chat_ctx.dispatch(ChatDispatch::UpdateCurrentChat(chat.clone()));
                    navigation.push(&MeRoute::ChatRoom { chat_id: chat.id });
                    force_toggle("modal_overlay");
            }
            });
        })
    };

    let reset = {
        let entity = entity.clone();
        let name = name.clone();
        let selected_friends = selected_friends.clone();
        Callback::from(move |x: ()| {
            let entity = entity.clone();

            if let Some(chat) = entity {

                name.set(chat.1.clone());
            } else {
                name.set(String::new());
            }
            selected_friends.set(Vec::<FullFriend>::new());
        })
    };

    html! {
        <Modal
            modal_class={"chat_modal"}
            button_class={"chat"}
            button_icon={button_icon}
            reset_state={reset}
        >
            <>
                {
                    if new {
                        html!{<>
                            <h2>{ "Start a New Chat" }</h2>
                            <p>{"Add up to 9 more friends"}</p>
                        </>}
                    } else {
                        html!{
                            <h2>{ "Edit Chat Name" }</h2>
                        }
                    }
                }
                <form onsubmit={on_submit}>
                    <label>
                        {"Chat Name:"}
                        <input
                            type="text"
                            onchange={on_change}
                            placeholder="Chatroom name"
                            value={(*name).clone()}
                        />
                    </label>
                    { if new {
                        html!{
                            <label>
                                {"Members"}
                                <div name="friends" id="friends" multiple={true}>
                                {
                                    for member_ctx.friends.friends.clone().iter().map(|friend|{
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





                                                {friend.member.clone().avatar("friend-item", "30")}
                                                <span>{friend.member.username.clone()}</span>
                                            </div>
                                        }})
                               }
                                </div>
                            </label>
                        }
                    } else {
                        html!{}
                    }}
                    <button type="submit" style="margin-top: 10px;">{ if new { "Create" } else { "Save" } }</button>
                </form>
            </>
        </Modal>
    }
}