use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement};
use yew::platform::spawn_local;
use yew::prelude::*;
use crate::comps::modal::Modal;
use crate::models::member::{Member, MemberShort, MultiMember};
use crate::models::server::{MultiServer, Server};
use crate::utils::api_requests::api_get;
use crate::views::home::me::chat_menu::search_item::SearchItem;

#[function_component(SearchModal)]
pub fn search_modal() -> Html{
    let search_ref = use_node_ref();
    let member_state = use_state(|| MultiMember { members: Vec::<MemberShort>::new() });
    let server_state = use_state(|| MultiServer { servers: Vec::<Server>::new() });

    let on_keyup = {
        let search_ref = search_ref.clone();
        let member_state = member_state.clone();
        let server_state = server_state.clone();

        Callback::from(move |event: KeyboardEvent| {
            let search_ref = search_ref.clone();
            let member_state = member_state.clone();
            let server_state = server_state.clone();

            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                let search_term = input.value();
                spawn_local(async move {
                    if let Ok(members) = api_get::<MultiMember>(format!("/member/search?search_term={}", &search_term)).await {
                        member_state.set(members);
                    }
                    if let Ok(servers) = api_get::<MultiServer>(format!("/server/search?search_term={}", &search_term)).await {
                        server_state.set(servers);
                    }
                });
            }
        })
    };
    let reset = {
        let member_state = member_state.clone();
        let server_state = server_state.clone();

        Callback::from(move |x: ()| {
            let member_state = member_state.clone();
            let server_state = server_state.clone();

            member_state.set(MultiMember { members: Vec::<MemberShort>::new() });
            server_state.set(MultiServer { servers: Vec::<Server>::new() });
        })
    };

    html!{
        <Modal
            modal_class="search_modal"
            button_class="search"
            button_icon={html!({"Find or start a conversation"})}
            reset_state={reset}
        >
            <input
                id="search"
                type="search"
                ref={search_ref}
                placeholder={"Where would you like to go?"}
                onkeyup={on_keyup}
                autofocus={true}
            />
            <div>
                {
                    if (*member_state).members.clone().is_empty() {
                        html!{}
                    } else {
                        html!{<>
                            <h2>{"Members"}</h2>
                            <ul>
                                {
                                    for (*member_state).members.clone().into_iter().map(|member|{
                                        html!{
                                            <SearchItem
                                                name={member.username.clone()}
                                                second={member.username}
                                            />
                                        }
                                    })
                                }
                            </ul>
                            <hr/>
                        </>}
                    }
                }
                {
                    if (*member_state).members.clone().is_empty() {
                        html!{}
                    } else {
                        html!{<>
                            <h2>{"Servers"}</h2>
                            <ul>
                                {
                                    for (*server_state).servers.clone().into_iter().map(|server|{
                                        html!{
                                            <SearchItem
                                                name={server.name}
                                                second={server.created_at.format("%Y-%m-%d %H:%M:%S").to_string()}
                                            />
                                        }
                                    })
                                }
                            </ul>
                        </>}
                    }
                }
            </div>
        </Modal>
    }
}
