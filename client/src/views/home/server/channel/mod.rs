use gloo::net::http::Request;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlTextAreaElement, InputEvent, KeyboardEvent, ScrollBehavior};
use yew::{Callback, function_component, Html, html, use_context, use_effect_with, use_node_ref};
use crate::utils::auth_token;
use crate::comps::icon::get_random_svg;
use crate::contexts::server_context::{get_channel_thread, ServerDispatch, TServerContext};
use crate::contexts::member_context::TMemberContext;

pub(crate) mod welcome;
pub(crate) mod new_channel;
pub(crate) mod edit_server;

#[function_component(ChannelComp)]
pub fn channel() -> Html {
    let member_ctx = use_context::<TMemberContext>().unwrap();
    let server_ctx = use_context::<TServerContext>().unwrap();
    let channel_ref = use_node_ref();

    {
        let channel_ref = channel_ref.clone();
        let posts = server_ctx.current_thread.posts.clone();
        use_effect_with(posts, move |_| {
            if let Some(channel_el) = channel_ref.cast::<web_sys::Element>() {
                channel_el.set_scroll_top(channel_el.scroll_height());
            }
            || ()
        });
    }

    let textarea_ref = use_node_ref();
    let on_submit = {
        let textarea_ref = textarea_ref.clone();
        let member_ctx = member_ctx.clone();
        let server_ctx = server_ctx.clone();

        move || {
            let member_ctx = member_ctx.clone();
            let server_ctx = server_ctx.clone();
            let message: HtmlTextAreaElement = textarea_ref.cast::<HtmlTextAreaElement>().unwrap();
            let mut form_data = vec![
                ("content", message.value().clone()),
                ("author_id", member_ctx.member.id.to_string()),
                ("channel_id", server_ctx.current_channel.id.to_string()),
            ];
            spawn_local(async move {
                let request = Request::post("http://localhost:8000/post")
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .header("Authorization", &auth_token())
                    .body(form_data.iter()
                        .map(|(key, value)| format!("{}={}", key, urlencoding::encode(value)))
                        .collect::<Vec<String>>()
                        .join("&")
                    )
                    .unwrap()
                    .send()
                    .await;

                match request {
                    Ok(response) => {
                        if response.ok() {
                            message.set_value("");
                            server_ctx.dispatch(ServerDispatch::UpdateChannelThread(get_channel_thread(server_ctx.current_channel.id).await.unwrap()));
                        } else { log::error!("Request failed with status: {}", response.status()); }
                    }
                    Err(err) => { log::error!("Failed to send request: {:?}", err); }
                }
            });
        }
    };

    html! {
        <div id="main-content" class="server-channel">
            <div id="channel-header">
                <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" fill="currentColor" class="bi bi-hash" viewBox="0 0 16 16">
                    <path d="M8.39 12.648a1 1 0 0 0-.015.18c0 .305.21.508.5.508.266 0 .492-.172.555-.477l.554-2.703h1.204c.421 0 .617-.234.617-.547 0-.312-.188-.53-.617-.53h-.985l.516-2.524h1.265c.43 0 .618-.227.618-.547 0-.313-.188-.524-.618-.524h-1.046l.476-2.304a1 1 0 0 0 .016-.164.51.51 0 0 0-.516-.516.54.54 0 0 0-.539.43l-.523 2.554H7.617l.477-2.304c.008-.04.015-.118.015-.164a.51.51 0 0 0-.523-.516.54.54 0 0 0-.531.43L6.53 5.484H5.414c-.43 0-.617.22-.617.532s.187.539.617.539h.906l-.515 2.523H4.609c-.421 0-.609.219-.609.531s.188.547.61.547h.976l-.516 2.492c-.008.04-.015.125-.015.18 0 .305.21.508.5.508.265 0 .492-.172.554-.477l.555-2.703h2.242zm-1-6.109h2.266l-.515 2.563H6.859l.532-2.563z"/>
                </svg>
                {&server_ctx.current_channel.name}
                <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" fill="currentColor" class="bi bi-bell-slash-fill" viewBox="0 0 16 16">
                    <path d="M5.164 14H15c-1.5-1-2-5.902-2-7q0-.396-.06-.776zm6.288-10.617A5 5 0 0 0 8.995 2.1a1 1 0 1 0-1.99 0A5 5 0 0 0 3 7c0 .898-.335 4.342-1.278 6.113zM10 15a2 2 0 1 1-4 0zm-9.375.625a.53.53 0 0 0 .75.75l14.75-14.75a.53.53 0 0 0-.75-.75z"/>
                </svg>
                <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" fill="currentColor" class="bi bi-pin-angle-fill" viewBox="0 0 16 16">
                    <path d="M9.828.722a.5.5 0 0 1 .354.146l4.95 4.95a.5.5 0 0 1 0 .707c-.48.48-1.072.588-1.503.588-.177 0-.335-.018-.46-.039l-3.134 3.134a6 6 0 0 1 .16 1.013c.046.702-.032 1.687-.72 2.375a.5.5 0 0 1-.707 0l-2.829-2.828-3.182 3.182c-.195.195-1.219.902-1.414.707s.512-1.22.707-1.414l3.182-3.182-2.828-2.829a.5.5 0 0 1 0-.707c.688-.688 1.673-.767 2.375-.72a6 6 0 0 1 1.013.16l3.134-3.133a3 3 0 0 1-.04-.461c0-.43.108-1.022.589-1.503a.5.5 0 0 1 .353-.146"/>
                </svg>
                <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" fill="currentColor" class="bi bi-people-fill" viewBox="0 0 16 16">
                    <path d="M7 14s-1 0-1-1 1-4 5-4 5 3 5 4-1 1-1 1zm4-6a3 3 0 1 0 0-6 3 3 0 0 0 0 6m-5.784 6A2.24 2.24 0 0 1 5 13c0-1.355.68-2.75 1.936-3.72A6.3 6.3 0 0 0 5 9c-4 0-5 3-5 4s1 1 1 1zM4.5 8a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5"/>
                </svg>
                <input type="text" placeholder="Search"/>
                <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" fill="currentColor" class="bi bi-inbox-fill" viewBox="0 0 16 16">
                    <path d="M4.98 4a.5.5 0 0 0-.39.188L1.54 8H6a.5.5 0 0 1 .5.5 1.5 1.5 0 1 0 3 0A.5.5 0 0 1 10 8h4.46l-3.05-3.812A.5.5 0 0 0 11.02 4zm-1.17-.437A1.5 1.5 0 0 1 4.98 3h6.04a1.5 1.5 0 0 1 1.17.563l3.7 4.625a.5.5 0 0 1 .106.374l-.39 3.124A1.5 1.5 0 0 1 14.117 13H1.883a1.5 1.5 0 0 1-1.489-1.314l-.39-3.124a.5.5 0 0 1 .106-.374z"/>
                </svg>
                <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" fill="currentColor" class="bi bi-question-circle-fill" viewBox="0 0 16 16">
                    <path d="M16 8A8 8 0 1 1 0 8a8 8 0 0 1 16 0M5.496 6.033h.825c.138 0 .248-.113.266-.25.09-.656.54-1.134 1.342-1.134.686 0 1.314.343 1.314 1.168 0 .635-.374.927-.965 1.371-.673.489-1.206 1.06-1.168 1.987l.003.217a.25.25 0 0 0 .25.246h.811a.25.25 0 0 0 .25-.25v-.105c0-.718.273-.927 1.01-1.486.609-.463 1.244-.977 1.244-2.056 0-1.511-1.276-2.241-2.673-2.241-1.267 0-2.655.59-2.75 2.286a.237.237 0 0 0 .241.247m2.325 6.443c.61 0 1.029-.394 1.029-.927 0-.552-.42-.94-1.029-.94-.584 0-1.009.388-1.009.94 0 .533.425.927 1.01.927z"/>
                </svg>
            </div>
            <hr/>
            <section id="channel_chat_room" ref={channel_ref}>
                <svg xmlns="http://www.w3.org/2000/svg" width="50" height="50" fill="currentColor" class="bi bi-hash" viewBox="0 0 16 16">
                    <path d="M8.39 12.648a1 1 0 0 0-.015.18c0 .305.21.508.5.508.266 0 .492-.172.555-.477l.554-2.703h1.204c.421 0 .617-.234.617-.547 0-.312-.188-.53-.617-.53h-.985l.516-2.524h1.265c.43 0 .618-.227.618-.547 0-.313-.188-.524-.618-.524h-1.046l.476-2.304a1 1 0 0 0 .016-.164.51.51 0 0 0-.516-.516.54.54 0 0 0-.539.43l-.523 2.554H7.617l.477-2.304c.008-.04.015-.118.015-.164a.51.51 0 0 0-.523-.516.54.54 0 0 0-.531.43L6.53 5.484H5.414c-.43 0-.617.22-.617.532s.187.539.617.539h.906l-.515 2.523H4.609c-.421 0-.609.219-.609.531s.188.547.61.547h.976l-.516 2.492c-.008.04-.015.125-.015.18 0 .305.21.508.5.508.265 0 .492-.172.554-.477l.555-2.703h2.242zm-1-6.109h2.266l-.515 2.563H6.859l.532-2.563z"/>
                </svg>
                <h1>
                    {"Welcome to "}{&server_ctx.current_channel.name}
                </h1>
                <p>
                    {format!("This is the start of the #{} channel.", &server_ctx.current_channel.name)}
                </p>
                <hr/>
                {for server_ctx.current_thread.posts.clone().into_iter().rev().map(|post| html!{
                    <div>
                        {get_random_svg(true, "user-icon", "16")}
                        <div>
                            <p>{post.author.username.clone()}<span>{post.created_at()}</span></p>
                            <p>{&post.content}</p>
                        </div>
                    </div>
                })}
            </section>
            <div id="channel-input">
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-plus-circle-fill" viewBox="0 0 16 16">
                    <path d="M16 8A8 8 0 1 1 0 8a8 8 0 0 1 16 0M8.5 4.5a.5.5 0 0 0-1 0v3h-3a.5.5 0 0 0 0 1h3v3a.5.5 0 0 0 1 0v-3h3a.5.5 0 0 0 0-1h-3z"/>
                </svg>
                <textarea
                    name="post"
                    placeholder={format!("Post #{}", &server_ctx.current_channel.name)}
                    wrap="hard"
                    rows={2}
                    cols={60}
                    ref={textarea_ref.clone()}
                    oninput={Callback::from(|e: InputEvent| {
                        e.prevent_default();
                        let target = e.target().unwrap();
                        let textarea: HtmlTextAreaElement = target.dyn_into::<HtmlTextAreaElement>().unwrap();

                        // Reset the height to auto to allow shrinking
                        textarea.style().set_property("height", "auto").unwrap();

                        // Set the height to the scrollHeight so it expands as needed
                        let scroll_height = textarea.scroll_height();
                        textarea.style().set_property("height", &format!("{}px", scroll_height)).unwrap();
                    })}
                    onkeydown={Callback::from(move |e: KeyboardEvent| {
                        // Check if the "Enter" key is pressed
                        if e.key() == "Enter" && !e.shift_key() {
                            e.prevent_default(); // Prevent the default behavior (new line)
                            on_submit();
                        }
                    })}
                >
                </textarea>
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-gift-fill" viewBox="0 0 16 16">
                    <path d="M3 2.5a2.5 2.5 0 0 1 5 0 2.5 2.5 0 0 1 5 0v.006c0 .07 0 .27-.038.494H15a1 1 0 0 1 1 1v1a1 1 0 0 1-1 1H1a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h2.038A3 3 0 0 1 3 2.506zm1.068.5H7v-.5a1.5 1.5 0 1 0-3 0c0 .085.002.274.045.43zM9 3h2.932l.023-.07c.043-.156.045-.345.045-.43a1.5 1.5 0 0 0-3 0zm6 4v7.5a1.5 1.5 0 0 1-1.5 1.5H9V7zM2.5 16A1.5 1.5 0 0 1 1 14.5V7h6v9z"/>
                </svg>
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-emoji-smile-fill" viewBox="0 0 16 16">
                    <path d="M8 16A8 8 0 1 0 8 0a8 8 0 0 0 0 16M7 6.5C7 7.328 6.552 8 6 8s-1-.672-1-1.5S5.448 5 6 5s1 .672 1 1.5M4.285 9.567a.5.5 0 0 1 .683.183A3.5 3.5 0 0 0 8 11.5a3.5 3.5 0 0 0 3.032-1.75.5.5 0 1 1 .866.5A4.5 4.5 0 0 1 8 12.5a4.5 4.5 0 0 1-3.898-2.25.5.5 0 0 1 .183-.683M10 8c-.552 0-1-.672-1-1.5S9.448 5 10 5s1 .672 1 1.5S10.552 8 10 8"/>
                </svg>
            </div>
        </div>
    }
}