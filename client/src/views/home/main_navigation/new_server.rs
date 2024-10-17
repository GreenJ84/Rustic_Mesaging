use gloo::net::http::Request;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, MouseEvent, SubmitEvent, window};
use yew::{Callback, function_component, Html, html, use_context, use_node_ref, use_state};
use yew_router::hooks::use_navigator;
use crate::contexts::member_context::{MemberDispatch, get_servers, TMemberContext};
use crate::models::server::{MultiServer, Server};
use crate::comps::modal::{force_toggle, Modal, toggle_modal};
use crate::contexts::server_context::{ServerDispatch, TServerContext};
use crate::utils::api_requests::{api_post, PostResponse};
use crate::views::home::HomeRoute;
use crate::views::home::me::MeRoute;

#[function_component(NewServerForm)]
pub fn new_server_modal() -> Html {
    let member_ctx = use_context::<TMemberContext>().unwrap();
    let server_ctx = use_context::<TServerContext>().unwrap();
    let navigation = use_navigator().unwrap();

    let name_ref = use_node_ref();
    let description_ref = use_node_ref();

    let on_submit = {
        let name_ref = name_ref.clone();
        let description_ref = description_ref.clone();
        let submit_nav = navigation.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let member_ctx = member_ctx.clone();
            let server_ctx = server_ctx.clone();
            let submit_nav = submit_nav.clone();

            let name = name_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            let description = description_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            let mut form_data = vec![
                ("name".to_string(), name.clone()),
                ("description".to_string(), description.clone()),
                ("icon".to_string(), String::new()),
                ("owner_id".to_string(), member_ctx.member.id.clone().to_string())
            ];
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(request) = api_post::<Server>(String::from("server"), form_data, false).await {
                    if let PostResponse::NonResponse(server) = request {
                        force_toggle("modal_overlay");
                        member_ctx.dispatch(MemberDispatch::UpdateServers(get_servers().await.unwrap()));
                        server_ctx.dispatch(ServerDispatch::UpdateServer(server.clone()));
                        submit_nav.push(&HomeRoute::Server { server_id: server.id });
                    }
                }
            });
        })
    };

    html! {
        <Modal
            modal_class={"new_server_modal"}
            button_class={"create_server"}
            button_icon={html!(
                <>
                    <span></span>
                    <svg xmlns="http://www.w3.org/2000/svg" width="26" height="26" fill="currentColor" class="bi bi-patch-plus" viewBox="0 0 16 16">
                        <path fill-rule="evenodd" d="M8 5.5a.5.5 0 0 1 .5.5v1.5H10a.5.5 0 0 1 0 1H8.5V10a.5.5 0 0 1-1 0V8.5H6a.5.5 0 0 1 0-1h1.5V6a.5.5 0 0 1 .5-.5"/>
                        <path d="m10.273 2.513-.921-.944.715-.698.622.637.89-.011a2.89 2.89 0 0 1 2.924 2.924l-.01.89.636.622a2.89 2.89 0 0 1 0 4.134l-.637.622.011.89a2.89 2.89 0 0 1-2.924 2.924l-.89-.01-.622.636a2.89 2.89 0 0 1-4.134 0l-.622-.637-.89.011a2.89 2.89 0 0 1-2.924-2.924l.01-.89-.636-.622a2.89 2.89 0 0 1 0-4.134l.637-.622-.011-.89a2.89 2.89 0 0 1 2.924-2.924l.89.01.622-.636a2.89 2.89 0 0 1 4.134 0l-.715.698a1.89 1.89 0 0 0-2.704 0l-.92.944-1.32-.016a1.89 1.89 0 0 0-1.911 1.912l.016 1.318-.944.921a1.89 1.89 0 0 0 0 2.704l.944.92-.016 1.32a1.89 1.89 0 0 0 1.912 1.911l1.318-.016.921.944a1.89 1.89 0 0 0 2.704 0l.92-.944 1.32.016a1.89 1.89 0 0 0 1.911-1.912l-.016-1.318.944-.921a1.89 1.89 0 0 0 0-2.704l-.944-.92.016-1.32a1.89 1.89 0 0 0-1.912-1.911z"/>
                    </svg>
                </>
            )}
        >
            <>
                <h2>{ "Create a New Server" }</h2>
                <form onsubmit={on_submit.clone()} >
                    <label for="name">
                        { "Name:" }
                        <input
                            ref={name_ref}
                            type="text"
                            id="name"
                            name="name"
                            required=true
                        />
                    </label>
                    <br/>
                    <label for="description">
                        { "Description:" }
                        <textarea
                            ref={description_ref}
                            id="description"
                            name="description"
                            required=true
                        ></textarea>
                    </label>
                    <br/>
                    <button type="submit" style="margin-top: 10px;">{ "Create" }</button>
                </form>
            </>
        </Modal>
    }
}