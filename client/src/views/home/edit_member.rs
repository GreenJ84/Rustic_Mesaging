
use web_sys::SubmitEvent;
use yew::{Callback, function_component, Html, html, NodeRef, Properties};
use crate::comps::modal::Modal;

#[derive(Properties, PartialEq)]
pub struct EditProps {
    pub(crate) first_node_ref: NodeRef,
    pub(crate) first_label: String,
    pub(crate) second_node_ref: NodeRef,
    pub(crate) on_submit: Callback<SubmitEvent>
}

#[function_component(EditMemberModal)]
pub fn edit_member(EditProps {
    first_node_ref,
    first_label,
    second_node_ref,
    on_submit
}: &EditProps) -> Html {
    html! {
        <Modal
            modal_class={"edit_member_modal"}
            button_class={"edit_member"}
            button_icon={html!(
                {"Edit"}
            )}
        >
            <>
                <h2>{format!("Update your {}", first_label)}</h2>
                <form onsubmit={on_submit.clone()} >
                    <label for={first_label.clone()}>
                        { format!("{}:", first_label.clone()) }
                        <input
                            ref={first_node_ref}
                            type="text"
                            id={first_label.clone().to_lowercase()}
                            name={first_label.clone().to_lowercase()}
                            pattern="[a-zA-Z0-9]{3, 80}"
                            required=true
                        />
                    </label>
                    <br/>
                    <label for={"confirm_password"}>
                        { "Confirm current password" }
                        <input
                            ref={second_node_ref}
                            type="text"
                            id={"current_password"}
                            name={"current_password"}
                            pattern="[a-zA-Z0-9]{3, 80}"
                            required=true
                        />
                    </label>
                    <br/>
                    <button type="submit" style="margin-top: 10px;">{ "Create" }</button>
                </form>
            </>
        </Modal>
    }
}