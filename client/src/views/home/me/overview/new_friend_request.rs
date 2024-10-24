use gloo::net::http::Request;
use web_sys::{HtmlElement, HtmlInputElement};
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use crate::comps::main_navigation::discover_servers::DiscoverServersModal;
use crate::models::member::{Member, MemberShort};
use crate::models::server::Server;
use crate::comps::modal::toggle_modal;
use crate::comps::modal_portal::ModalPortal;
use crate::views::home::HomeRoute;
use crate::contexts::member_context::{MemberDispatch, get_friends, get_requests, get_servers, TMemberContext};
use crate::utils::api_requests::{api_get, api_post, PostResponse};

#[function_component(NewFriendRequest)]
pub fn new_friend_request() -> Html {
    let app_ctx = use_context::<TMemberContext>().unwrap();
    let navigation = use_navigator().unwrap();
    let receiver = use_state(|| MemberShort::default());
    let is_modal_open = use_state(|| false);

    let username_state = use_state(|| String::new());
    let username_error = use_state(|| false);
    let note_ref = use_node_ref();

    let on_change = {
        let username_state = username_state.clone();
        let username_error = username_error.clone();

        Callback::from(move |event: InputEvent| {
            if *username_error {
                username_error.set(false);
            }
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                username_state.set(input.value());
            }
        })
    };

    let on_search_submit = {
        let app_ctx = app_ctx.clone();
        let is_modal_open = is_modal_open.clone();
        let receiver = receiver.clone();
        let username_state = username_state.clone();
        let username_error = username_error.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let username = username_state.clone();
            let username_error = username_error.clone();
            if (*username_state).is_empty() {
                return;
            }
            let app_ctx = app_ctx.clone();
            let is_modal_open = is_modal_open.clone();
            let receiver = receiver.clone();


            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(member) = api_get::<MemberShort>(format!("member/{}", (*username))).await {
                    receiver.set(member);
                    toggle_modal("modal_overlay");
                    is_modal_open.set(!*is_modal_open);
                } else  {
                    username_error.set(true);
                }
            });
        })
    };

    let on_send = {
        let app_ctx = app_ctx.clone();
        let submit_nav = navigation.clone();
        let receiver = receiver.clone();
        let note_ref = note_ref.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let app_ctx = app_ctx.clone();
            let submit_nav = submit_nav.clone();
            let receiver = receiver.clone();
            let note = note_ref.cast::<web_sys::HtmlTextAreaElement>().unwrap().value();
            let mut form_data = vec![
                (String::from("sender_id"), app_ctx.member.id.to_string()),
                (String::from("receiver_id"), receiver.id.to_string()),
                (String::from("note"), note.clone()),
            ];
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(res) = api_post::<()>(String::from("request"), form_data, true).await {
                    if let PostResponse::Response(_, response) = res {
                        match response.status(){
                            201 => {
                                toggle_modal("modal_overlay");
                                app_ctx.dispatch(MemberDispatch::UpdateRequests(get_requests().await.unwrap()));
                            },
                            200 => {
                                toggle_modal("modal_overlay");
                                app_ctx.dispatch(MemberDispatch::UpdateRequests(get_requests().await.unwrap()));
                                app_ctx.dispatch(MemberDispatch::UpdateFriends(get_friends().await.unwrap()));
                            },
                            _ => {
                                log::error!("Failed to process response");
                            }
                        }
                    }
                }
            });
        })
    };

    let toggle_modal = {
        let is_modal_open = is_modal_open.clone();
        Callback::from(move |event: MouseEvent| {
            toggle_modal("modal_overlay");
            is_modal_open.set(!*is_modal_open);
        })
    };


    html! {
        <section>
            {
                if *is_modal_open {
                    html!{<ModalPortal onclick={toggle_modal.clone()}>
                        <div class={"modal new_friend_request"}>
                            <button class="return" onclick={toggle_modal.clone()}>
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-x-lg" viewBox="0 0 16 16">
                                    <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
                                </svg>
                            </button>
                            <h2>{ "Add a Note" }</h2>
                            <form onsubmit={on_send}>
                                <label for="note">
                                    <textarea
                                        ref={note_ref}
                                        id="note"
                                        name="note"
                                    >
                                    </textarea>
                                </label>
                                <button type="submit">{ "Send" }</button>
                            </form>
                        </div>
                    </ModalPortal>}
                } else { html!{} }
    }

            <h2>{ "Add Friend" }</h2>
            <p>{"Add a friend using their RTC Username"}</p>
            <form onsubmit={on_search_submit} >
                <label for="name">
                    <input
                        oninput={on_change}
                        type="text"
                        id="name"
                        name="name"
                        placeholder="Add friends by their username."
                        autocomplete="off"
                        required=true
                    />
                </label>
                <button type="submit" disabled={(*username_state).is_empty()}>{ "Send Friend Request" }</button>
                { if *username_error { html!{ <span>{"Username not found."}</span> } } else { html!{} }}
            </form>
            <hr/>
            <h3>{"Other Places to make friends"}</h3>
            <DiscoverServersModal
                switch_icon={html!(<>
                    <svg fill="currentColor" enable-background="new 0 0 512 512" viewBox="0 0 512 512" width="45" height="45" xmlns="http://www.w3.org/2000/svg">
                        <g><g>
                            <path d="m255.5 226.2c-16.9 0-30 13.1-29.9 29.8.3 16.8 13.9 30.2 30.7 30.3 16.2.1 29.3-13 29.4-29.2 0-.2 0-.4 0-.6.2-16.6-13-30.2-29.6-30.4-.2.1-.4.1-.6.1z"/>
                            <path d="m256 0c-141.4 0-256 114.6-256 256s114.6 256 256 256 256-114.6 256-256-114.6-256-256-256zm135.6 144.5c-21.8 56.1-43.9 112-65.8 168-2.2 6.1-6.9 10.8-12.9 13.1-56.1 22-112.3 44.1-168.4 66.2-1.9.7-3.8 1.2-5.7 1.6-15 .1-23.4-12.8-18.3-26 11-28.5 22.3-56.9 33.5-85.3 10.8-27.4 21.5-54.7 32.2-82.2 2.5-6.5 6.6-11.1 13.1-13.6 55.8-21.8 111.6-43.7 167.4-65.7 12.1-4.8 23.3 0 26 11.6.7 4.1.4 8.4-1.1 12.3z"/>
                        </g></g>
                    </svg>
                    <span>{"Explore Discoverable Servers"}</span>
                    <svg fill="currentColor" viewBox="0 0 6.3499999 6.3500002" width="32" height="32" xmlns="http://www.w3.org/2000/svg">
                        <g id="layer1" transform="translate(0 -290.65)">
                            <path id="path9429" d="m2.2580394 291.96502a.26460982.26460982 0 0 0 -.1741496.46871l1.6190225 1.38699-1.6190225 1.38648a.26460982.26460982 0 1 0 .3436483.40049l1.8536335-1.58595a.26460982.26460982 0 0 0 0-.40256l-1.8536335-1.5875a.26460982.26460982 0 0 0 -.1694987-.0667z" font-variant-ligatures="normal" font-variant-position="normal" font-variant-caps="normal" font-variant-numeric="normal" font-variant-alternates="normal" font-feature-settings="normal" text-indent="0" text-align="start" text-decoration-line="none" text-decoration-style="solid" text-decoration-color="rgb(0,0,0)" text-transform="none" text-orientation="mixed" white-space="normal" shape-padding="0" isolation="auto" mix-blend-mode="normal" solid-color="rgb(0,0,0)" solid-opacity="1" vector-effect="none"/>
                        </g>
                    </svg>
                </>)}
            />
        </section>
    }
}