use yew::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, MouseEvent, window};

use crate::comps::modal_portal::ModalPortal;

#[derive(Properties, PartialEq)]
pub struct Props{
    pub modal_class: &'static str,
    pub children: Html,
    pub button_class: &'static str,
    pub button_icon: Html,
}

pub fn toggle_modal(id: &str){
    if let Some(document) = window().and_then(|w| w.document()) {
        if let Some(overlay) = document.get_element_by_id(id) {
            if let Some(html_element) = overlay.dyn_ref::<HtmlElement>() {
                html_element.class_list().toggle("closed").unwrap();
            }
        }
    }
}

#[function_component(Modal)]
pub fn modal(Props {
     modal_class,
     button_class,
     children,
     button_icon,
 }: &Props) -> Html {
    let is_modal_open = use_state(|| false);

    let toggle_modal = {
        let is_modal_open = is_modal_open.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            event.prevent_default();
            toggle_modal("modal_overlay");
            is_modal_open.set(!*is_modal_open);
        })
    };


    html! {
        <>
            {
                if *is_modal_open {
                    html!{<ModalPortal onclick={toggle_modal.clone()}>
                            <div
                                class={format!("modal {}", *modal_class)}
                                onclick={Callback::from(move |e: MouseEvent| {
                                    e.prevent_default();
                                    e.stop_propagation();
                                })}
                            >
                                <button class="return" onclick={toggle_modal.clone()}>
                                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-x-lg" viewBox="0 0 16 16">
                                        <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
                                    </svg>
                                </button>
                                {children.clone()}
                            </div>
                    </ModalPortal>}
                } else { html!{ } }
            }
            <button onclick={toggle_modal} title={*button_class} class={*button_class}>
                {button_icon.clone()}
            </button>
        </>
    }
}