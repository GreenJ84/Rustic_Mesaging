use yew::prelude::*;
use yew::html::ChildrenProps;
use std::rc::Rc;
use gloo::utils::window;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

use crate::models::{
    member::Member,
    server::MultiServer,
    friends::MultiFriend,
    friend_request::MultiFriendRequest
};
use crate::utils::api_requests::{api_get, api_head};

pub type TMemberContext = UseReducerHandle<MemberContext>;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MemberContext {
    pub logged_in: bool,
    pub member: Member,
    pub servers: MultiServer,
    pub friends: MultiFriend,
    pub requests: MultiFriendRequest,
}
pub enum MemberDispatch {
    UpdateMember(Member),
    UpdateServers(MultiServer),
    UpdateFriends(MultiFriend),
    UpdateRequests(MultiFriendRequest)
}
impl Reducible for MemberContext {
    type Action = MemberDispatch;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let new_context = match action {
            MemberDispatch::UpdateMember(member) => Self {
                logged_in: !member.eq(&Member::default()),
                member,
                servers: MultiServer::default(),
                friends:  MultiFriend::default(),
                requests: MultiFriendRequest::default(),
            },
            MemberDispatch::UpdateServers(servers) => Self {
                logged_in: self.logged_in,
                member: self.member.clone(),
                servers,
                friends: self.friends.clone(),
                requests: self.requests.clone()
            },
            MemberDispatch::UpdateFriends(friends) => Self {
                logged_in: self.logged_in,
                member: self.member.clone(),
                servers: self.servers.clone(),
                friends,
                requests: self.requests.clone()
            },
            MemberDispatch::UpdateRequests(requests) => Self {
                logged_in: self.logged_in,
                member: self.member.clone(),
                servers: self.servers.clone(),
                friends: self.friends.clone(),
                requests
            },
        };
        Rc::new(new_context)
    }
}

#[function_component(MemberContextProvider)]
pub fn member_context_provider(props: &ChildrenProps) -> Html {
    let context = use_reducer(|| MemberContext {
        logged_in: false,
        member: Member::default(),
        servers: MultiServer::default(),
        friends: MultiFriend::default(),
        requests: MultiFriendRequest { incoming: Vec::new(), outgoing: Vec::new() }
    });

    {
        let context = context.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if is_logged_in().await{
                    if let Ok(member) = get_member().await {
                        context.dispatch(MemberDispatch::UpdateMember(member));
                    }
                };
            })
        });
    }

    {
        let context = context.clone();
        use_effect_with(context.member.clone(), move |_| {
            if context.member.clone().eq(&Member::default()){
                return;
            }
            let context = context.clone();
            spawn_local(async move {
                if let Ok(servers) = get_servers().await {
                    context.dispatch(MemberDispatch::UpdateServers(servers));
                }

                if let Ok(friends) = get_friends().await {
                    context.dispatch(MemberDispatch::UpdateFriends(friends));
                }

                if let Ok(requests) = get_requests().await {
                    context.dispatch(MemberDispatch::UpdateRequests(requests));
                }
            });
        });
    }

    html! {
        <ContextProvider<TMemberContext> context={context}>
            {props.children.clone()}
        </ContextProvider<TMemberContext>>
    }
}

pub async fn is_logged_in() -> bool {
    if let Ok(Some(token)) = window().local_storage().unwrap().unwrap().get_item("jwt_token") {
        return !token.is_empty() && validate_token().await;
    }
    false
}

async fn validate_token() -> bool{
    api_head("member".to_string()).await
}

async fn get_member() -> Result<Member, ()> {
    api_get::<Member>("member".to_string()).await
}

pub async fn get_servers() -> Result<MultiServer, ()> {
    api_get::<MultiServer>("member/servers".to_string()).await
}

pub async fn get_friends() -> Result<MultiFriend, ()>{
    api_get::<MultiFriend>("member/friends".to_string()).await
}

pub async fn get_requests() -> Result<MultiFriendRequest, ()>{
    api_get::<MultiFriendRequest>("member/requests".to_string()).await
}