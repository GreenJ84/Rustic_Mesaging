use gloo::net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use crate::comps::icon::Icon;
use crate::models::friends::FullFriend;
use crate::comps::modal::Modal;
use crate::contexts::member_context::{MemberDispatch, get_friends, TMemberContext};
use crate::utils::api_requests::api_delete;
use crate::utils::format_date;

#[derive(Properties, PartialEq)]
pub struct Props{
    pub(crate) friend: FullFriend
}

#[function_component(FriendItem)]
pub fn friend_menu_item(Props { friend }: &Props) -> Html {
    let member_ctx = use_context::<TMemberContext>().unwrap();

    let handle_delete = {
        let member_ctx = member_ctx.clone();
        let friend = friend.clone();

        Callback::from(move |e: MouseEvent| {
            let member_ctx = member_ctx.clone();
            let friend = friend.clone();

            spawn_local(async move {
                if let Ok(_) = api_delete(format!("friend/{}",
                     friend.member.id.to_string()
                )).await {
                    member_ctx.dispatch(MemberDispatch::UpdateFriends(get_friends().await.unwrap()));
                }
            });
        })
    };

    html!{
        <li class="overview_menu_item">
            {friend.member.clone().avatar("", "40")}
            <div>
                <h4>{&friend.member.username} <span>{&friend.member.username}</span></h4>
                <p class="created">{"Friends since: "}{format_date(&friend.created_at)}</p>
            </div>
            <div>
                <button
                    title="Message Friend"
                    class="message"
                >
                    <Icon
                        class_name="icon"
                        size="20"
                        avatar={ html!{
                            <>
                               <g><g>
                                    <path d="M438.957,19.478H73.043C32.766,19.478,0,52.245,0,92.521V339.48c0,40.276,32.766,73.043,73.043,73.043h28.663l0.561,64.483c0.05,5.943,3.463,11.344,8.809,13.942c2.172,1.056,4.512,1.575,6.84,1.575c3.399,0,6.773-1.106,9.565-3.261l99.424-76.737h212.052c40.276,0,73.043-32.767,73.043-73.043V92.521C512,52.245,479.233,19.478,438.957,19.478z"/>
                                </g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g>
                            </>
                        }}
                        color={"currentColor"}
                        ratio={"0 0 512 512"}
                    />
                </button>
                <button
                    title="Delete Friend"
                    class="delete"
                    onclick={handle_delete}
                >
                    <Icon
                        class_name="icon"
                        size="20"
                        avatar={ html!{
                            <path d="m194.800781 164.769531 128.210938-128.214843c8.34375-8.339844 8.34375-21.824219 0-30.164063-8.339844-8.339844-21.824219-8.339844-30.164063 0l-128.214844 128.214844-128.210937-128.214844c-8.34375-8.339844-21.824219-8.339844-30.164063 0-8.34375 8.339844-8.34375 21.824219 0 30.164063l128.210938 128.214843-128.210938 128.214844c-8.34375 8.339844-8.34375 21.824219 0 30.164063 4.15625 4.160156 9.621094 6.25 15.082032 6.25 5.460937 0 10.921875-2.089844 15.082031-6.25l128.210937-128.214844 128.214844 128.214844c4.160156 4.160156 9.621094 6.25 15.082032 6.25 5.460937 0 10.921874-2.089844 15.082031-6.25 8.34375-8.339844 8.34375-21.824219 0-30.164063zm0 0"/>
                        }}
                        color={"currentColor"}
                        ratio={"0 0 329.26933 329"}
                    />
                </button>
            </div>
        </li>
    }
}