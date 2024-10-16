use std::rc::Rc;
use gloo::net::http::Request;
use yew::prelude::*;
use yew::html::ChildrenProps;
use serde::{Deserialize, Serialize};
use crate::models::channel::{Channel, MultiChannel};
use crate::models::post::{MultiPost, Post};
use crate::models::server::Server;
use crate::utils::api_requests::api_get;

pub type TServerContext = UseReducerHandle<ServerContext>;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ServerContext {
    pub current_server: Server,
    pub channels: MultiChannel,
    pub current_channel: Channel,
    pub current_thread: MultiPost
}
pub enum ServerDispatch {
    UpdateServer(Server),
    UpdateServerChannels(MultiChannel),
    UpdateChannel(Channel),
    UpdateChannelThread(MultiPost),
}
impl Reducible for ServerContext {
    type Action = ServerDispatch;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let new_context = match action {
            ServerDispatch::UpdateServer(server) => {
                Self{
                    current_server: server,
                    channels: MultiChannel::default(),
                    current_channel: Channel::default(),
                    current_thread: MultiPost::default()
                }
            },
            ServerDispatch::UpdateServerChannels(channels) => {
                Self{
                    current_server: self.current_server.clone(),
                    channels,
                    current_channel: self.current_channel.clone(),
                    current_thread: self.current_thread.clone(),
                }
            },
            ServerDispatch::UpdateChannel(channel) => {
                Self{
                    current_server: self.current_server.clone(),
                    channels: self.channels.clone(),
                    current_channel: channel,
                    current_thread: self.current_thread.clone()
                }
            },
            ServerDispatch::UpdateChannelThread(posts) => {
                Self{
                    current_server: self.current_server.clone(),
                    channels: self.channels.clone(),
                    current_channel: self.current_channel.clone(),
                    current_thread: posts
                }
            }
        };
        Rc::new(new_context)
    }
}

// Create a context hook
#[function_component(ServerProvider)]
pub fn server_provider(props: &ChildrenProps) -> Html {
    let context = use_reducer(|| ServerContext {
        current_server: Server::default(),
        channels: MultiChannel::default(),
        current_channel: Channel::default(),
        current_thread: MultiPost::default()
    });

    {
        let context = context.clone();
        use_effect_with( context.current_server.clone(), move |_| {
            if context.current_server.clone() == Server::default() {
                return;
            }
            let context = context.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(channels) = get_server_channels(context.current_server.id.clone()).await {
                    context.dispatch(ServerDispatch::UpdateServerChannels(channels));
                }
            });
        });
    }

    {
        let context = context.clone();
        let channel_id = context.current_channel.id;
        use_effect_with(channel_id, move |_| {
            if context.current_channel.clone() == Channel::default() {
                return;
            }
            let context = context.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(thread) = get_channel_thread(context.current_channel.id.clone()).await {
                    context.dispatch(ServerDispatch::UpdateChannelThread(thread));
                }
            });
        });
    }


    html! {
        <ContextProvider<TServerContext> context={context}>
            {props.children.clone()}
        </ContextProvider<TServerContext>>
    }
}

pub async fn get_server(server_id: i32) -> Result<Server, ()> {
     api_get::<Server>(format!("server/{}", server_id)).await
}

pub async fn get_server_channels(server_id: i32) -> Result<MultiChannel, ()> {
    api_get::<MultiChannel>(format!("server/{}/channels", server_id)).await
}

pub async fn get_channel_thread(channel_id: i32) -> Result<MultiPost, ()> {
    api_get::<MultiPost>(format!("channel/{}/posts", channel_id)).await
}