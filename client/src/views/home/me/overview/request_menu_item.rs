use chrono::NaiveDateTime;
use gloo::net::http::Request;
use yew::platform::spawn_local;
use yew::prelude::*;
use crate::models::friend_request::RequestDetailed;
use crate::models::member::Member;
use crate::comps::icon::{get_random_svg, Icon};
use crate::comps::modal::Modal;
use crate::views::home::me::overview::edit_friend_request::EditFriendRequest;
use crate::contexts::member_context::{MemberDispatch, get_friends, get_requests, TMemberContext};
use crate::utils::api_requests::api_delete;

#[derive(Debug, Clone, PartialEq)]
pub enum Version {
    Pending,
    Incoming,
}
impl Version{
    pub fn value(self: &Self) -> String{
        match self{
            Version::Pending => "pending".to_string(),
            Version::Incoming => "incoming".to_string()
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct Props{
    pub(crate) request: RequestDetailed,
    pub(crate) version: Version
}

#[function_component(RequestItem)]
pub fn request_menu_item(Props { request, version }: &Props) -> Html {
    let member_ctx = use_context::<TMemberContext>().unwrap();

    let handle_request = |option: &str, is_sender: bool| {
        let member_ctx = member_ctx.clone();
        let request = request.clone();
        let version = version.clone();
        let option = option.to_string();

        Callback::from(move |e: MouseEvent| {
            let member_ctx = member_ctx.clone();
            let request = request.clone();
            let version = version.clone();
            let option = option.clone();
            spawn_local(async move {
                if let Ok(_) = api_delete(format!("request/{}/{}/{}",
                    if is_sender {member_ctx.member.id} else {request.member.id},
                    if is_sender {request.member.id} else {member_ctx.member.id},
                    option.clone()
                )).await {
                    let requests = get_requests().await.unwrap();
                    member_ctx.dispatch(MemberDispatch::UpdateRequests(requests));
                    if option.eq("accept"){
                        let friends = get_friends().await.unwrap();
                        member_ctx.dispatch(MemberDispatch::UpdateFriends(friends));
                    }
                }
            });
        })
    };
    let delete_request = handle_request("deny", true);
    let accept_request = handle_request("accept", false);
    let deny_request = handle_request("deny", false);


    html!{
        <li class={format!("overview_menu_item {}", version.value())} >
            {get_random_svg(true, "friend-icon", "30")}
            <h4>{&request.member.username} <span>{&request.member.username}</span></h4>
            {if let Some(note) = &request.note {
                html!{ <p>{note}</p> }
            } else { html!{} } }
            {match version{
                Version::Pending => html!{ <>
                        <p>{"Outgoing Friend Request"}</p>
                        <p>{format!("Sent: {:?}", &request.created_at)}</p>
                        <button
                            title="Delete Friend Request"
                            onclick={delete_request}
                        >
                            <Icon
                                class_name="icon"
                                size="30"
                                avatar={ html!{
                                    <path d="m194.800781 164.769531 128.210938-128.214843c8.34375-8.339844 8.34375-21.824219 0-30.164063-8.339844-8.339844-21.824219-8.339844-30.164063 0l-128.214844 128.214844-128.210937-128.214844c-8.34375-8.339844-21.824219-8.339844-30.164063 0-8.34375 8.339844-8.34375 21.824219 0 30.164063l128.210938 128.214843-128.210938 128.214844c-8.34375 8.339844-8.34375 21.824219 0 30.164063 4.15625 4.160156 9.621094 6.25 15.082032 6.25 5.460937 0 10.921875-2.089844 15.082031-6.25l128.210937-128.214844 128.214844 128.214844c4.160156 4.160156 9.621094 6.25 15.082032 6.25 5.460937 0 10.921874-2.089844 15.082031-6.25 8.34375-8.339844 8.34375-21.824219 0-30.164063zm0 0"/>
                                }}
                                color={"red"}
                                ratio={"0 0 329.26933 329"}
                            />
                        </button>
                        <EditFriendRequest request={request.clone()} />
                    </>},
                Version::Incoming =>
                    html!{ <>
                        <p>{"Incoming Friend Request"}</p>
                        <button
                            title="Accept Friend Request"
                            onclick={accept_request}
                        >
                            <Icon
                                class_name="icon"
                                size="30"
                                avatar={ html!{
                                    <>
                                        <g><g>
                                        <path d="M504.502,75.496c-9.997-9.998-26.205-9.998-36.204,0L161.594,382.203L43.702,264.311c-9.997-9.998-26.205-9.997-36.204,0c-9.998,9.997-9.998,26.205,0,36.203l135.994,135.992c9.994,9.997,26.214,9.99,36.204,0L504.502,111.7C514.5,101.703,514.499,85.494,504.502,75.496z"/>
                                        </g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g><g></g>
                                    </>
                                }}
                                color={"green"}
                                ratio={"0 0 512 512"}
                            />
                        </button>
                        <button
                            title="Deny Friend Request"
                            onclick={deny_request}
                        >
                            <Icon
                                class_name="icon"
                                size="30"
                                avatar={ html!{
                                    <path d="m194.800781 164.769531 128.210938-128.214843c8.34375-8.339844 8.34375-21.824219 0-30.164063-8.339844-8.339844-21.824219-8.339844-30.164063 0l-128.214844 128.214844-128.210937-128.214844c-8.34375-8.339844-21.824219-8.339844-30.164063 0-8.34375 8.339844-8.34375 21.824219 0 30.164063l128.210938 128.214843-128.210938 128.214844c-8.34375 8.339844-8.34375 21.824219 0 30.164063 4.15625 4.160156 9.621094 6.25 15.082032 6.25 5.460937 0 10.921875-2.089844 15.082031-6.25l128.210937-128.214844 128.214844 128.214844c4.160156 4.160156 9.621094 6.25 15.082032 6.25 5.460937 0 10.921874-2.089844 15.082031-6.25 8.34375-8.339844 8.34375-21.824219 0-30.164063zm0 0"/>
                                }}
                                color={"red"}
                                ratio={"0 0 329.26933 329"}
                            />
                        </button>
                    </>}
                }
            }
        </li>
    }
}