use chrono::NaiveDateTime;
use yew::prelude::*;
use crate::models::member::{Member, MemberShort};
use crate::models::server::Server;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub(crate) icon: Html,
    pub(crate) name: String,
    pub(crate) second: String
}
#[function_component(SearchItem)]
pub fn search_item(Props { name, second, icon }: &Props) -> Html{

    html!{
        <li class="search_item">
            {icon.clone()}
            <p>
                {name}
                <span>{second}</span>
            </p>
        </li>
    }
}