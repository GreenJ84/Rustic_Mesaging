use gloo::net::http::Request;
use web_sys::{MouseEvent, SubmitEvent, window};
use yew::{Callback, function_component, Html, html, Properties, use_context, use_node_ref, use_ref};
use yew::platform::spawn_local;
use yew_router::hooks::use_navigator;
use yew_router::navigator::Navigator;
use crate::AppRoute;
use crate::models::member::Member;
use crate::models::server::Server;
use crate::comps::icon::get_random_svg;
use crate::comps::modal::{Modal, toggle_modal};
use crate::views::home::edit_member::EditMemberModal;
use crate::views::home::HomeRoute;
use crate::contexts::member_context::{MemberDispatch, get_friends, get_requests, get_servers, TMemberContext};
use crate::utils::api_requests::{api_delete, api_put};

#[function_component(Profile)]
pub fn profile() -> Html {
    let member_ctx = use_context::<TMemberContext>().unwrap();
    let navigation = use_navigator().unwrap();

    let toggle_profile = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        e.stop_propagation();
        toggle_modal("profile-container");
    });


    let logout = {
        let navigation = navigation.clone();

        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            let navigation: Navigator = navigation.clone();
            if let Some(window) = window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    match storage.remove_item("jwt_token") {
                        Ok(_) => {
                            log::info!("JWT token erased successfully from local storage.");
                            let _ = window.location().reload();
                        }
                        Err(err) => log::error!("Failed to erase JWT token: {:?}", err),
                    }
                } else { log::error!("Could not access local storage."); }
            } else { log::error!("No window object available."); }
        })
    };

    let delete_account = {
        let logout = logout.clone();
        Callback::from(move |e: MouseEvent| {
            let logout = logout.clone();
            spawn_local(async move {
                if let Ok(_) = api_delete(String::from("member")).await {
                    logout.emit(e);
                }
            });
        })
    };

    let member_ctx_clone = member_ctx.clone();
    let update = move |data: Vec<(String, String)>, is_password: bool| {
        let member_ctx = member_ctx_clone.clone();

        spawn_local(async move {
            if let Ok(member) = api_put::<Member>(format!("member{}", if is_password {"/password"} else {""}), data).await {
                member_ctx.dispatch(MemberDispatch::UpdateMember(member));
            }
        });
    };

    let new_username_ref = use_node_ref();
    let new_email_ref = use_node_ref();
    let new_password_ref = use_node_ref();
    let old_password_ref = use_node_ref();

    let update_username = {
        let update = update.clone();
        let member_ctx = member_ctx.clone();
        let new_username_ref = new_username_ref.clone();
        let old_password_ref = old_password_ref.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let update = update.clone();
            let member_ctx = member_ctx.clone();
            let username = new_username_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            let password = old_password_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            update(vec![
                (String::from("username"), username),
                (String::from("password"), password.clone()),
                (String::from("confirm_password"), password),
                (String::from("email"), member_ctx.member.email.clone())
            ], false)
        })
    };
    let update_password = {
        let update = update.clone();
        let member_ctx = member_ctx.clone();
        let new_password_ref = new_password_ref.clone();
        let old_password_ref = old_password_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let update = update.clone();
            let member_ctx = member_ctx.clone();
            let password = new_password_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            let old_password = old_password_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            update(vec![
                (String::from("username"), member_ctx.member.username.clone()),
                (String::from("password"), password),
                (String::from("confirm_password"), old_password),
                (String::from("email"), member_ctx.member.email.clone())
            ], true)
        })
    };
    let update_email = {
        let update = update.clone();
        let member_ctx = member_ctx.clone();
        let new_email_ref = new_email_ref.clone();
        let old_password_ref = old_password_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let update = update.clone();
            let member_ctx = member_ctx.clone();
            let email = new_email_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            let password = old_password_ref.cast::<web_sys::HtmlInputElement>().unwrap().value();
            update(vec![
                (String::from("username"), member_ctx.member.username.clone()),
                (String::from("password"), password.clone()),
                (String::from("confirm_password"), password),
                (String::from("email"), email)
            ], false)
        })
    };

    html! {
        <main id="profile-container" class="closed">
             <button class="return" onclick={toggle_profile.clone()}>
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-x-lg" viewBox="0 0 16 16">
                    <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
                </svg>
            </button>
            <div class="profile-chat_menu">
                <input type="search" placeholder="search"/>

                <h4>{"User Settings"}</h4>
                <ul>{
                    vec!["My Account", "Profiles", "Privacy & Safety", "Family Center", "Authorized Apps", "Devices", "Connections", "Clips", "Friend Requests"]
                        .into_iter().map(|title|
                            html! {<li title={title}>{title}</li>}
                    ).collect::<Html>()
                }</ul>

                <h4>{"Billing Settings"}</h4>
                <ul>{
                    vec!["Nitro", "Server Boost", "Subscriptions", "Gift Inventory", "Billing"]
                        .into_iter().map(|title|
                            html! {<li title={title}>{title}</li>}
                    ).collect::<Html>()
                }</ul>

                <h4>{"App Settings"}</h4>
                <ul>{
                    vec!["Appearance", "Accessibility", "Voice & Video", "Chat", "Notifications", "Keybinds", "Language", "Streamer Mode", "Advanced"]
                        .into_iter().map(|title|
                            html! {<li title={title}>{title}</li>}
                    ).collect::<Html>()
                }</ul>

                <h4>{"Activity Settings"}</h4>
                <ul>
                    <li title="Activity Privacy">{"Activity Privacy"}</li>
                </ul>

                <ul>{
                    vec!["What's New", "Merch", "HypeSquad"]
                        .into_iter().map(|title|
                            html! {<li title={title}>{title}</li>}
                    ).collect::<Html>()
                }</ul>
                <ul>
                    <li title="Logout" onclick={logout.clone()}>
                        {"Logout"}
                        <span>
                            <svg viewBox="0 0 512.016 512" width="20" height="20" xmlns="http://www.w3.org/2000/svg">
                                <path d="m496 240.007812h-202.667969c-8.832031 0-16-7.167968-16-16 0-8.832031 7.167969-16 16-16h202.667969c8.832031 0 16 7.167969 16 16 0 8.832032-7.167969 16-16 16zm0 0"/>
                                <path d="m416 320.007812c-4.097656 0-8.191406-1.558593-11.308594-4.691406-6.25-6.253906-6.25-16.386718 0-22.636718l68.695313-68.691407-68.695313-68.695312c-6.25-6.25-6.25-16.382813 0-22.632813 6.253906-6.253906 16.386719-6.253906 22.636719 0l80 80c6.25 6.25 6.25 16.382813 0 22.632813l-80 80c-3.136719 3.15625-7.230469 4.714843-11.328125 4.714843zm0 0"/>
                                <path d="m170.667969 512.007812c-4.566407 0-8.898438-.640624-13.226563-1.984374l-128.386718-42.773438c-17.46875-6.101562-29.054688-22.378906-29.054688-40.574219v-384c0-23.53125 19.136719-42.6679685 42.667969-42.6679685 4.5625 0 8.894531.6406255 13.226562 1.9843755l128.382813 42.773437c17.472656 6.101563 29.054687 22.378906 29.054687 40.574219v384c0 23.53125-19.132812 42.667968-42.664062 42.667968zm-128-480c-5.867188 0-10.667969 4.800782-10.667969 10.667969v384c0 4.542969 3.050781 8.765625 7.402344 10.28125l127.785156 42.582031c.917969.296876 2.113281.46875 3.480469.46875 5.867187 0 10.664062-4.800781 10.664062-10.667968v-384c0-4.542969-3.050781-8.765625-7.402343-10.28125l-127.785157-42.582032c-.917969-.296874-2.113281-.46875-3.476562-.46875zm0 0"/>
                                <path d="m325.332031 170.675781c-8.832031 0-16-7.167969-16-16v-96c0-14.699219-11.964843-26.667969-26.664062-26.667969h-240c-8.832031 0-16-7.167968-16-16 0-8.832031 7.167969-15.9999995 16-15.9999995h240c32.363281 0 58.664062 26.3046875 58.664062 58.6679685v96c0 8.832031-7.167969 16-16 16zm0 0"/>
                                <path d="m282.667969 448.007812h-85.335938c-8.832031 0-16-7.167968-16-16 0-8.832031 7.167969-16 16-16h85.335938c14.699219 0 26.664062-11.96875 26.664062-26.667968v-96c0-8.832032 7.167969-16 16-16s16 7.167968 16 16v96c0 32.363281-26.300781 58.667968-58.664062 58.667968zm0 0"/>
                            </svg>
                        </span>
                    </li>
                </ul>
                <ul id="social"></ul>
            </div>
            <div class="profile-details">
                <div class="profile-info-container">
                    <div></div>
                    {get_random_svg(true, "profile-avatar", "60")}
                    <h1>{member_ctx.member.username.clone()}</h1>
                    <div>
                        <h3>{"Display Name"}</h3>
                        <p>{member_ctx.member.username.clone()}</p>
                        <EditMemberModal
                            first_node_ref={new_username_ref.clone()}
                            first_label="Username"
                            second_node_ref={old_password_ref.clone()}
                            on_submit={update_username.clone()}
                        />
                    </div>
                    <div>
                        <h3>{"Username"}</h3>
                        <p>{member_ctx.member.username.clone()}</p>
                        <EditMemberModal
                            first_node_ref={new_username_ref.clone()}
                            first_label="Username"
                            second_node_ref={old_password_ref.clone()}
                            on_submit={update_username}
                        />
                    </div>
                    <div>
                        <h3>{"Email"}</h3>
                        <p>{member_ctx.member.email.clone()}</p>
                        <EditMemberModal
                            first_node_ref={new_email_ref.clone()}
                            first_label="Email"
                            second_node_ref={old_password_ref.clone()}
                            on_submit={update_email}
                        />
                    </div>
                </div>

                <h2>{"Password and Authentication"}</h2>
                <EditMemberModal
                    first_node_ref={new_password_ref.clone()}
                    first_label="Password"
                    second_node_ref={old_password_ref.clone()}
                    on_submit={update_password}
                />

                <h4>{"Authenticator App"}</h4>
                <p>{"Protect you Real-Time Chat App account with an extra layer of security."}</p>
                <button>{"Enable Authenticator App"}</button>

                <h4>{"Security Keys"}</h4>
                <p>{"Add an additional layer of protection to your account."}</p>
                <button>{"Register a Security Key"}</button>

                <h4>{"Account Removal"}</h4>
                <p>{"Disabling your account means you can recover it any time after taking the action."}</p>
                <button onclick={logout}>{"Disable Account"}</button>
                <button onclick={delete_account}>{"Delete Account Account"}</button>
            </div>
        </main>
    }
}