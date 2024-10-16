use yew::prelude::*;
use yew_router::prelude::*;
use gloo::net::http::Request;
use gloo::utils::window;
use web_sys::SubmitEvent;

use crate::contexts::member_context::{MemberDispatch, TMemberContext};
use crate::AppRoute;
use crate::models::member::Member;
use crate::utils::api_requests::{api_post, PostResponse};
use crate::views::{
    landing::LandingRoute,
    home::me::MeRoute
};

#[function_component(Login)]
pub fn login_modal() -> Html {
    let member_ctx = use_context::<TMemberContext>().unwrap();
    let navigation = use_navigator().unwrap();

    let username_ref = use_node_ref();
    let password_ref = use_node_ref();

    let on_submit = {
        let username_ref = username_ref.clone();
        let password_ref = password_ref.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let member_ctx = member_ctx.clone();
            let navigation = navigation.clone();

            let username = username_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            let password = password_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            let mut form_data = vec![
                ("username".to_string(), username.clone()),
                ("password".to_string(), password.clone()),
            ];
            wasm_bindgen_futures::spawn_local(async move {
                let request = api_post::<Member>("login".to_string(), form_data, true).await;
                match request {
                    Ok(post_response) => {
                        if let PostResponse::Response(member, response) = post_response{
                            if let Some(auth_header) = response.headers().get("authorization") {
                                if let Some(bearer_token) = auth_header.strip_prefix("Bearer ") {
                                    if let Err(_) = web_sys::window()
                                        .unwrap()
                                        .local_storage()
                                        .unwrap()
                                        .unwrap()
                                        .set_item("jwt_token", bearer_token)
                                    {
                                        log::error!("Failed to save authentication");
                                    } else {
                                        log::info!("Login successful.");
                                        // Save member
                                        member_ctx.dispatch(MemberDispatch::UpdateMember(member));
                                        navigation.push(&MeRoute::Overview)
                                    }
                                }
                            } else {
                                log::error!("Authorization header not found in response");
                            }
                        }
                    },
                    Err(_) => { log::error!("Login request failed"); }
                }
            });
        })
    };

    html! {
        <div class="modal-overlay">
            <div class="modal login-modal">
                <Link<AppRoute> to={AppRoute::Landing}>
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-x-lg" viewBox="0 0 16 16">
                        <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
                    </svg>
                </Link<AppRoute>>
                <Link<LandingRoute> to={LandingRoute::Register}>
                    {"Not registered?"}
                </Link<LandingRoute>>
                <h2>{ "Login" }</h2>
                <form onsubmit={on_submit}>
                    <label for="username">
                        { "Username:" }
                        <input ref={username_ref} type="text" id="username" name="username" required=true />
                    </label>
                    <br/>
                    <label for="password">
                        { "Password:" }
                        <input ref={password_ref} type="password" id="password" name="password" required=true />
                    </label>
                    <br/>
                    <button type="submit" style="margin-top: 10px;">{ "Login" }</button>
                </form>
            </div>
        </div>
    }
}