pub(crate) mod edit_member;
mod member_activity_report;

use gloo::net::http::Request;
use web_sys::{MouseEvent, SubmitEvent, window};
use yew::{Callback, function_component, Html, html, Properties, use_context, use_node_ref, use_ref};
use yew::platform::spawn_local;
use yew_router::hooks::use_navigator;
use yew_router::navigator::Navigator;
use crate::AppRoute;
use crate::models::member::Member;
use crate::models::server::Server;
use crate::comps::modal::{force_toggle, Modal, toggle_modal};
use crate::comps::profile::edit_member::EditMemberModal;
use crate::comps::profile::member_activity_report::MemberActivityReport;
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
                force_toggle("modal_overlay");
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
            <div class="profile-navigation">
                <div>
                    <input type="search" placeholder="Search"/>
                    <svg version="1.1" xmlns="http://www.w3.org/2000/svg" width="20" height="20" stroke="currentColor" fill="currentColor" viewBox="0 0 56.966 56.966" style="enable-background:new 0 0 56.966 56.966" >
                        <path d="M55.146,51.887L41.588,37.786c3.486-4.144,5.396-9.358,5.396-14.786c0-12.682-10.318-23-23-23s-23,10.318-23,23s10.318,23,23,23c4.761,0,9.298-1.436,13.177-4.162l13.661,14.208c0.571,0.593,1.339,0.92,2.162,0.92c0.779,0,1.518-0.297,2.079-0.837C56.255,54.982,56.293,53.08,55.146,51.887z M23.984,6c9.374,0,17,7.626,17,17s-7.626,17-17,17s-17-7.626-17-17S14.61,6,23.984,6z"/>
                        <g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g>
                    </svg>
                </div>
                <h4>{"USER SETTINGS"}</h4>
                <ul>{
                    vec!["My Account", "Profiles", "Privacy & Safety", "Family Center", "Authorized Apps", "Devices", "Connections", "Clips", "Friend Requests"]
                        .into_iter().map(|title|
                            html! {<li title={title}>{title}</li>}
                    ).collect::<Html>()
                }</ul>
                <hr/>
                <h4>{"Billing Settings"}</h4>
                <ul>{
                    vec!["Nitro", "Server Boost", "Subscriptions", "Gift Inventory", "Billing"]
                        .into_iter().map(|title|
                            html! {<li title={title}>{title}</li>}
                    ).collect::<Html>()
                }</ul>
                <hr/>
                <h4>{"APP SETTINGS"}</h4>
                <ul>{
                    vec!["Appearance", "Accessibility", "Voice & Video", "Chat", "Notifications", "Keybinds", "Language", "Streamer Mode", "Advanced"]
                        .into_iter().map(|title|
                            html! {<li title={title}>{title}</li>}
                    ).collect::<Html>()
                }</ul>
                <hr/>
                <h4>{"ACTIVITY SETTINGS"}</h4>
                <ul>
                    <li><MemberActivityReport /></li>
                    <li title="Activity Privacy">{"Activity Privacy"}</li>
                </ul>
                <hr/>
                <ul>{
                    vec!["What's New", "Merch", "HypeSquad"]
                        .into_iter().map(|title|
                            html! {<li title={title}>{title}</li>}
                    ).collect::<Html>()
                }</ul>
                <hr/>
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
                <hr/>
                <ul id="social">
                    <li>
                        <a tabindex="-1" class="anchor_af404b anchorUnderlineOnHover_af404b link_c44e94" href="https://twitter.com/discord" rel="noreferrer noopener" target="_blank" title="Twitter">
                            <svg aria-hidden="true" role="img" xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="none" viewBox="0 0 24 24">
                                <path fill="currentColor" d="M13.86 10.47 21.15 2h-1.73l-6.33 7.35L8.04 2H2.22l7.64 11.12L2.22 22h1.72l6.68-7.77L15.96 22h5.82l-7.92-11.53Zm-2.36 2.75-.78-1.11L4.57 3.3h2.65l4.97 7.11.77 1.1 6.46 9.25h-2.65l-5.27-7.54Z" class="foreground_c44e94"></path>
                            </svg>
                        </a>
                    </li>
                    <li>
                        <a tabindex="-1" class="anchor_af404b anchorUnderlineOnHover_af404b link_c44e94" href="https://www.instagram.com/discord/" rel="noreferrer noopener" target="_blank" title="Instagram">
                            <svg aria-hidden="true" role="img" xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="none" viewBox="0 0 24 24">
                                <path fill="currentColor" fill-rule="evenodd" d="M7 12a5 5 0 1 1 10 0 5 5 0 0 1-10 0Zm5-3a3 3 0 1 0 0 6 3 3 0 0 0 0-6Z" clip-rule="evenodd" class="foreground_c44e94"></path>
                                <path fill="currentColor" d="M17.25 8a1.25 1.25 0 1 0 0-2.5 1.25 1.25 0 0 0 0 2.5Z" class="foreground_c44e94"></path>
                                <path fill="currentColor" fill-rule="evenodd" d="M7.86 2.07a7.3 7.3 0 0 0-2.43.47A4.9 4.9 0 0 0 3.66 3.7a4.9 4.9 0 0 0-1.15 1.77 7.35 7.35 0 0 0-.46 2.43C2.01 8.96 2 9.3 2 12.02c0 2.71.02 3.06.07 4.12.05 1.07.22 1.8.47 2.43.26.66.6 1.21 1.16 1.77.55.55 1.11.9 1.77 1.15.64.24 1.36.41 2.43.46 1.06.04 1.4.05 4.12.05 2.71 0 3.06-.02 4.12-.07a6.14 6.14 0 0 0 4.2-1.63 6.15 6.15 0 0 0 1.6-4.2c.05-1.06.06-1.4.06-4.12 0-2.71-.02-3.05-.07-4.12a6.15 6.15 0 0 0-1.63-4.2 6.14 6.14 0 0 0-4.2-1.6C15.04 2 14.7 2 11.98 2c-2.71 0-3.05.02-4.12.07Zm.1 2c-.88.04-1.39.17-1.8.33a2.9 2.9 0 0 0-1.08.7 2.9 2.9 0 0 0-.7 1.09c-.16.4-.29.92-.33 1.8A68.6 68.6 0 0 0 4 12.01c0 2.7.02 3 .07 4.03.04.87.17 1.38.33 1.79.17.42.36.73.7 1.08.35.34.67.54 1.09.7.41.16.92.29 1.8.33 1.01.04 1.32.05 4.03.05 2.7 0 3-.02 4.03-.07 1.28-.06 2.23-.4 2.87-1.04.64-.64.97-1.6 1.03-2.87.04-1.02.05-1.33.05-4.04 0-2.7-.02-3-.07-4.03-.06-1.28-.4-2.23-1.04-2.87-.64-.64-1.6-.97-2.87-1.03A69.44 69.44 0 0 0 11.98 4c-2.7 0-3 .02-4.02.07Z" clip-rule="evenodd" class="foreground_c44e94"></path>
                            </svg>
                        </a>
                    </li>
                    <li>
                        <a tabindex="-1" class="anchor_af404b anchorUnderlineOnHover_af404b link_c44e94" href="https://www.facebook.com/discord/" rel="noreferrer noopener" target="_blank" title="Facebook">
                            <svg aria-hidden="true" role="img" xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="none" viewBox="0 0 24 24">
                                <path fill="currentColor" d="M23 12a11 11 0 1 0-12.72 10.87v-7.69h-2.8V12h2.8V9.58c0-2.76 1.64-4.28 4.16-4.28 1.2 0 2.46.21 2.46.21v2.7H15.5c-1.37 0-1.8.86-1.8 1.73V12h3.06l-.49 3.18h-2.56v7.69A11 11 0 0 0 23 12Z" class="foreground_c44e94"></path>
                            </svg>
                        </a>
                    </li>
                    <li>
                        <a tabindex="-1" class="anchor_af404b anchorUnderlineOnHover_af404b link_c44e94" href="https://www.youtube.com/discord/" rel="noreferrer noopener" target="_blank" title="YouTube">
                            <svg width="16" height="16" viewBox="0 0 24 24" aria-hidden="true" role="img">
                                <path fill-rule="evenodd" clip-rule="evenodd" d="M21.3766 4.10479C22.4093 4.38257 23.2225 5.20102 23.4985 6.24038C24 8.12411 24 12.0545 24 12.0545C24 12.0545 24 15.9848 23.4985 17.8688C23.2225 18.908 22.4093 19.7265 21.3766 20.0044C19.505 20.5091 12 20.5091 12 20.5091C12 20.5091 4.49496 20.5091 2.62336 20.0044C1.59082 19.7265 0.777545 18.908 0.501545 17.8688C0 15.9848 0 12.0545 0 12.0545C0 12.0545 0 8.12411 0.501545 6.24038C0.777545 5.20102 1.59082 4.38257 2.62336 4.10479C4.49496 3.59998 12 3.59998 12 3.59998C12 3.59998 19.505 3.59998 21.3766 4.10479ZM15.8182 12.0546L9.54551 15.623V8.48596L15.8182 12.0546Z" fill="currentColor" class="foreground_c44e94"></path>
                            </svg>
                        </a>
                    </li>
                    <li>
                        <a tabindex="-1" class="anchor_af404b anchorUnderlineOnHover_af404b link_c44e94" href="https://www.tiktok.com/@discord" rel="noreferrer noopener" target="_blank" title="TikTok">
                            <svg width="16" height="16" viewBox="0 0 24 24" aria-hidden="true" role="img">
                                <path class="foreground_c44e94" fill="currentColor" d="M17.836 6.009A4.794 4.794 0 0 1 15.658 2h-3.439l-.005 13.78a2.892 2.892 0 0 1-2.885 2.782 2.893 2.893 0 0 1-2.89-2.89 2.894 2.894 0 0 1 2.89-2.89c.298 0 .583.048.853.133v-3.51a6.308 6.308 0 0 0-.853-.062A6.336 6.336 0 0 0 3 15.672a6.324 6.324 0 0 0 2.702 5.181A6.29 6.29 0 0 0 9.329 22a6.336 6.336 0 0 0 6.329-6.329V8.683c1.348.968 3 1.539 4.784 1.539V6.783c-.96 0-1.855-.285-2.605-.775v.001Z"></path>
                            </svg>
                        </a>
                    </li>
                </ul>
            </div>
            <div class="profile-details">
                <button class="return" onclick={toggle_profile.clone()}>
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-x-lg" viewBox="0 0 16 16">
                        <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
                    </svg>
                </button>
                <h1>{"My Account"}</h1>
                <div class="profile-info-container">
                    <div class="banner"></div>
                    {member_ctx.member.clone().avatar("profile-avatar", "60")}
                    <h2>
                        {member_ctx.member.username.clone()}
                        <svg width="28" height="28" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" fill="currentColor">
                            <circle cx="12" cy="12" r="2"/>
                            <circle cx="4" cy="12" r="2"/>
                            <circle cx="20" cy="12" r="2"/>
                        </svg>
                    </h2>
                    <div class="info">
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
                <button class="disable-account" onclick={logout}>{"Disable Account"}</button>
                <button class="delete-account" onclick={delete_account}>{"Delete Account"}</button>
            </div>
        </main>
    }
}