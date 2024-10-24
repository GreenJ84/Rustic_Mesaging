use yew::prelude::*;
use web_sys::HtmlTextAreaElement;
use wasm_bindgen::JsCast;
use yew::platform::spawn_local;
use crate::comps::icon::Icon;
use crate::contexts::member_context::TMemberContext;
use crate::contexts::server_context::{get_channel_thread, ServerDispatch, TServerContext};
use crate::models::post::{Post, PostDetail};
use crate::utils::api_requests::api_put;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub post: PostDetail,
}

#[function_component(PostItem)]
pub fn post_item(Props { post }: &Props) -> Html {
    let member_ctx = use_context::<TMemberContext>().unwrap();
    let server_ctx = use_context::<TServerContext>().unwrap();

    let author = post.author.id.eq(&member_ctx.member.id);
    let editing = use_state(|| false);
    let content = use_state(|| post.content.clone());


    {
        let editing = editing.clone();
        use_effect(move || {
            || editing.set(false);
        });
    }

    {
        let server_ctx = server_ctx.clone();
        let content = content.clone();
        let post = post.clone();
        use_effect_with(server_ctx.current_thread.clone(), move |new_thread| {
            if let Some(updated_post) = new_thread.posts.iter().find(|p| p.id == post.id) {
                content.set(updated_post.content.clone());
            }
            || ()
        });
    }

    let on_edit_click = {
        let post = post.clone();
        let editing = editing.clone();
        Callback::from(move |event: MouseEvent| {
            if author{
                editing.set(!(*editing));
            }
        })
    };
    let on_cancel_click = {
        let post = post.clone();
        let content = content.clone();
        let on_edit_click = on_edit_click.clone();
        Callback::from(move |event: MouseEvent| {
            content.set(post.content.clone());
            on_edit_click.emit(event);
        })
    };

    let on_save_click = {
        let member_ctx = member_ctx.clone();
        let server_ctx = server_ctx.clone();
        let editing = editing.clone();
        let content = content.clone();
        let post = post.clone();

        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            let member_ctx = member_ctx.clone();
            let server_ctx = server_ctx.clone();
            let editing = editing.clone();
            let content = content.clone();
            let post = post.clone();

            spawn_local(async move {
                let editing = editing.clone();
                let content = content.clone();
                let mut form_data = vec![
                    (String::from("content"), (*content).clone()),
                    (String::from("author_id"), member_ctx.member.id.to_string()),
                    (String::from("channel_id"), server_ctx.current_channel.id.to_string()),
                ];

                if let Ok(_) = api_put::<Post>(format!("/post/{}", post.id), form_data).await {
                    if let Ok(thread) = get_channel_thread(server_ctx.current_channel.id).await {
                        editing.set(false);
                        server_ctx.dispatch(ServerDispatch::UpdateChannelThread(thread));
                    }
                }
            });
        })
    };
    let on_save_enter = {
        let on_save_click = on_save_click.clone();
        let on_cancel_click = on_cancel_click.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" && !e.shift_key() {
                on_save_click.emit(MouseEvent::new("click").unwrap());
            }
            if e.key() == "Escape" {
                on_cancel_click.emit(MouseEvent::new("click").unwrap());
            }
        })
    };

    let on_change = {
        let content = content.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlTextAreaElement = event.target().unwrap().dyn_into().unwrap();
            content.set(input.value());
        })
    };


    html! {
        <>
            <div
                class={format!("{}{}",
                    if author {"self"} else {"other"},
                    if (*editing) { " active" } else { " why" }
                )}
            >
                {post.author.clone().avatar("user-icon", "20")}
                <div>
                    <p>{&post.author.username}<span>{&post.created_at()}</span></p>
                    {
                        if *editing {
                            html! {
                                <>
                                    <textarea value={(*content).clone()} oninput={on_change} onkeydown={on_save_enter}></textarea>
                                    <button class="cancel" onclick={on_cancel_click}>{"Cancel"}</button>
                                    <button class="save" onclick={on_save_click}>{"Save"}</button>
                                </>
                            }
                        } else {
                            html! {
                                <>
                                    <p class="content">{&*content}</p>
                                </>
                            }
                        }
                    }
                </div>
                {
                    if author {
                        html!{
                            <button class="toggle" onclick={on_edit_click.clone()}>
                                <Icon
                                    class_name="icon"
                                    size="32"
                                    avatar={ html!{
                                        <>
                                            <g id="Layer_13" data-name="Layer 13" stroke="transparent" fill="currentColor">
                                                <path d="m16 30a14 14 0 1 1 14-14 14 14 0 0 1 -14 14zm0-26a12 12 0 1 0 12 12 12 12 0 0 0 -12-12zm0 17.05a2 2 0 1 0 2 2 2 2 0 0 0 -2-2zm0-7a2 2 0 1 0 2 2 2 2 0 0 0 -2-1.95zm0-7.05a2 2 0 1 0 2 2 2 2 0 0 0 -2-2z"/>
                                            </g>
                                        </>
                                    }}
                                    color={"black"}
                                    ratio={"0 0 32 32"}
                                />
                            </button>
                        }
                    } else { html!{} }
                }
            </div>
        </>
    }
}
