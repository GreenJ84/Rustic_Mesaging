use yew::prelude::*;
use web_sys::MouseEvent;
use wasm_bindgen::JsCast;
use gloo::events::EventListener;

#[derive(Properties, PartialEq)]
pub struct PortalProps {
    #[prop_or_default]
    pub children: Html,
    pub onclick: Callback<MouseEvent>
}

#[function_component(ModalPortal)]
pub fn modal_portal( PortalProps{ children, onclick }: &PortalProps) -> Html {
    let modal_host = gloo::utils::document()
        .get_element_by_id("modal_overlay")
        .expect("Expected to find a #modal_host element");

    {
        let onclick = onclick.clone();
        let modal_host = modal_host.clone();
        use_effect_with(
            (), move |_| {
                let listener = EventListener::new(&modal_host, "click", move |event| {
                    event.prevent_default();
                    event.stop_propagation();
                    onclick.emit(MouseEvent::new("click").unwrap());
                });

                || drop(listener)
            },
        );
    }

    create_portal(
        children.clone(),
        modal_host.into(),
    )
}