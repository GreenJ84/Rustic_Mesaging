
pub(crate) mod request_menu_item;
pub(crate) mod friend_menu_item;
pub(crate) mod new_friend_request;
pub(crate) mod edit_friend_request;

use serde::{Deserialize, Serialize};
use web_sys::MouseEvent;
use yew::{Callback, function_component, Html, html, use_context, use_state};

use crate::contexts::member_context::TMemberContext;
use crate::views::{
    home::me::overview::{
        friend_menu_item::FriendItem,
        new_friend_request::NewFriendRequest,
        request_menu_item::{RequestItem, Version}
    }
};

#[function_component(Overview)]
pub fn overview() -> Html {
    let app_ctx = use_context::<TMemberContext>().unwrap();
    let page_state = use_state(|| "online");

    html! {
        <main id="main-content" class="overview">
            <div id="overview-header">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" class="bi bi-person-raised-hand" viewBox="0 0 16 16">
                    <path d="M6 6.207v9.043a.75.75 0 0 0 1.5 0V10.5a.5.5 0 0 1 1 0v4.75a.75.75 0 0 0 1.5 0v-8.5a.25.25 0 1 1 .5 0v2.5a.75.75 0 0 0 1.5 0V6.5a3 3 0 0 0-3-3H6.236a1 1 0 0 1-.447-.106l-.33-.165A.83.83 0 0 1 5 2.488V.75a.75.75 0 0 0-1.5 0v2.083c0 .715.404 1.37 1.044 1.689L5.5 5c.32.32.5.754.5 1.207"/>
                    <path d="M8 3a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3"/>
                </svg>
                <p class="friend">{"Friends"}</p>
                <hr/>
                <button
                    onclick={let page_state= page_state.clone();
                        Callback::from(move|e: MouseEvent| {
                            e.prevent_default();
                            page_state.clone().set("online");
                        })}
                    class={if (*page_state).eq("online") {"active"} else {""} }
                >{"Online"}</button>
                <button
                    onclick={let page_state= page_state.clone();
                        Callback::from(move|e: MouseEvent| {
                            e.prevent_default();
                            page_state.clone().set("all");
                        })}
                    class={if (*page_state).eq("all") {"active"} else {""} }
                >{"All"}</button>
                <button
                    onclick={let page_state= page_state.clone();
                        Callback::from(move|e: MouseEvent| {
                            e.prevent_default();
                            page_state.clone().set("pending");
                        })}
                    class={if (*page_state).eq("pending") {"active"} else {""} }
                >{"Pending"}</button>
                <button
                    onclick={let page_state= page_state.clone();
                        Callback::from(move|e: MouseEvent| {
                            e.prevent_default();
                            page_state.clone().set("incoming");
                        })}
                    class={if (*page_state).eq("incoming") {"active"} else {""} }
                >{"Requests"}</button>
                <button
                    onclick={let page_state= page_state.clone();
                        Callback::from(move|e: MouseEvent| {
                            e.prevent_default();
                            page_state.clone().set("new");
                        })}
                >{"Add Friend"}</button>
                <hr/>
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" class="bi bi-inbox-fill" viewBox="0 0 16 16">
                    <path d="M4.98 4a.5.5 0 0 0-.39.188L1.54 8H6a.5.5 0 0 1 .5.5 1.5 1.5 0 1 0 3 0A.5.5 0 0 1 10 8h4.46l-3.05-3.812A.5.5 0 0 0 11.02 4zm-1.17-.437A1.5 1.5 0 0 1 4.98 3h6.04a1.5 1.5 0 0 1 1.17.563l3.7 4.625a.5.5 0 0 1 .106.374l-.39 3.124A1.5 1.5 0 0 1 14.117 13H1.883a1.5 1.5 0 0 1-1.489-1.314l-.39-3.124a.5.5 0 0 1 .106-.374z"/>
                </svg>
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" class="bi bi-question-circle-fill" viewBox="0 0 16 16">
                    <path d="M16 8A8 8 0 1 1 0 8a8 8 0 0 1 16 0M5.496 6.033h.825c.138 0 .248-.113.266-.25.09-.656.54-1.134 1.342-1.134.686 0 1.314.343 1.314 1.168 0 .635-.374.927-.965 1.371-.673.489-1.206 1.06-1.168 1.987l.003.217a.25.25 0 0 0 .25.246h.811a.25.25 0 0 0 .25-.25v-.105c0-.718.273-.927 1.01-1.486.609-.463 1.244-.977 1.244-2.056 0-1.511-1.276-2.241-2.673-2.241-1.267 0-2.655.59-2.75 2.286a.237.237 0 0 0 .241.247m2.325 6.443c.61 0 1.029-.394 1.029-.927 0-.552-.42-.94-1.029-.94-.584 0-1.009.388-1.009.94 0 .533.425.927 1.01.927z"/>
                </svg>
            </div>
            <hr/>

            <section id="overview-list">
                <div>
                    <input type="text" placeholder="Search"/>
                    <svg version="1.1" xmlns="http://www.w3.org/2000/svg" width="20" height="20" stroke="currentColor" fill="currentColor" viewBox="0 0 56.966 56.966" style="enable-background:new 0 0 56.966 56.966" >
                        <path d="M55.146,51.887L41.588,37.786c3.486-4.144,5.396-9.358,5.396-14.786c0-12.682-10.318-23-23-23s-23,10.318-23,23s10.318,23,23,23c4.761,0,9.298-1.436,13.177-4.162l13.661,14.208c0.571,0.593,1.339,0.92,2.162,0.92c0.779,0,1.518-0.297,2.079-0.837C56.255,54.982,56.293,53.08,55.146,51.887z M23.984,6c9.374,0,17,7.626,17,17s-7.626,17-17,17s-17-7.626-17-17S14.61,6,23.984,6z"/>
                        <g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g>
                    </svg>
                </div>
                {match *page_state{
                    "new" => html!{ <NewFriendRequest />},
                    "pending" => html! { <ul>
                        <li>{format!("PENDING Requests - {}", app_ctx.requests.outgoing.len())}</li>
                        {for app_ctx.requests.outgoing.clone().into_iter().map(|request| html! {
                            <RequestItem  request={request} version={Version::Pending} />
                        })}
                    </ul>},
                    "incoming" => html! { <ul>
                        <li>{format!("INCOMING Requests - {}", app_ctx.requests.incoming.len())}</li>
                        {for app_ctx.requests.incoming.clone().into_iter().map(|request| html! {
                            <RequestItem  request={request} version={Version::Incoming}/>
                        })}
                    </ul>},
                    "online" => html! { <ul>
                        <li>{format!("ONLINE - {}", app_ctx.friends.friends.len())}</li>
                        {for app_ctx.friends.friends.clone().into_iter().map(|friend| html! {
                            <FriendItem friend={friend} />
                        })}
                   </ul> },
                    _ => html! { <ul>
                        <li>{format!("ALL Friends - {}", app_ctx.friends.friends.len())}</li>
                        {for app_ctx.friends.friends.clone().into_iter().map(|friend| html! {
                            <FriendItem friend={friend} />
                        })}
                   </ul> }
                }}
            </section>
        </main>
    }
}