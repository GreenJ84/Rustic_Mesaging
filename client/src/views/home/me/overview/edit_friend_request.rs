use gloo::net::http::Request;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use crate::models::friend_request::RequestDetailed;
use crate::models::server::Server;
use crate::utils::auth_token;
use crate::comps::icon::Icon;
use crate::comps::modal::{Modal, toggle_modal};
use crate::views::home::HomeRoute;
use crate::views::home::me::overview::new_friend_request::NewFriendRequest;
use crate::contexts::member_context::{MemberDispatch, get_requests, get_servers, TMemberContext};

#[derive(Properties, PartialEq)]
pub struct Props{
    pub request: RequestDetailed
}

#[function_component(EditFriendRequest)]
pub fn edit_friend_request(Props { request }: &Props) -> Html{
    let app_ctx = use_context::<TMemberContext>().unwrap();
    let navigation = use_navigator().unwrap();

    let request = request.clone();
    let note_ref = use_node_ref();

    let on_submit = {
        let app_ctx = app_ctx.clone();
        let submit_nav = navigation.clone();
        let request = request.clone();
        let note_ref = note_ref.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let app_ctx = app_ctx.clone();
            let submit_nav = submit_nav.clone();
            let request = request.clone();
            let note = note_ref.cast::<web_sys::HtmlTextAreaElement>().unwrap().value();
            let mut form_data = vec![
                ("sender_id", app_ctx.member.id.to_string()),
                ("receiver_id", request.member.id.to_string()),
                ("note", note.clone()),
            ];
            wasm_bindgen_futures::spawn_local(async move {
                let request = Request::put("http://localhost:8000/request")
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .header("Authorization", &auth_token())
                    .body(form_data.into_iter()
                        .map(|(key, value)| format!("{}={}", key, urlencoding::encode(&value)))
                        .collect::<Vec<String>>()
                        .join("&")
                    )
                    .unwrap()
                    .send()
                    .await;

                match request {
                    Ok(response) => {
                        if response.ok() {
                            toggle_modal("edit_request_modal_overlay");
                            app_ctx.dispatch(MemberDispatch::UpdateRequests(get_requests().await.unwrap()))
                        } else { log::error!("Request failed with status: {}", response.status()); }
                    }
                    Err(err) => { log::error!("Failed to send request: {:?}", err); }
                }
            });
        })
    };

    html! {
        <Modal
            modal_class="edit_request_modal"
            button_class="edit_request"
            button_icon={html!{
                <Icon
                    class_name="icon"
                    size="30"
                    avatar={ html!{
                        <>
                            <g id="Layer_13" data-name="Layer 13">
                                <path d="m16 30a14 14 0 1 1 14-14 14 14 0 0 1 -14 14zm0-26a12 12 0 1 0 12 12 12 12 0 0 0 -12-12zm0 17.05a2 2 0 1 0 2 2 2 2 0 0 0 -2-2zm0-7a2 2 0 1 0 2 2 2 2 0 0 0 -2-1.95zm0-7.05a2 2 0 1 0 2 2 2 2 0 0 0 -2-2z"/>
                            </g>
                        </>
                    }}
                    color={"black"}
                    ratio={"0 0 32 32"}
                />
            }}
        >
            <>
                <h2>{ "Edit Request Note" }</h2>
                <form onsubmit={on_submit} >
                    <div for="name">
                        <textarea
                            ref={note_ref}
                            id="note"
                            name="note"
                        >
                        </textarea>
                        <button type="submit">{ "Save" }</button>
                    </div>
                </form>
            </>
        </Modal>
    }
}