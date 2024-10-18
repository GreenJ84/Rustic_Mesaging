use chrono::NaiveDateTime;
use yew::prelude::*;
use crate::comps::icon::get_random_svg;
use crate::models::member::{Member, MemberShort};
use crate::models::server::Server;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub(crate) name: String,
    pub(crate) second: String
}
#[function_component(SearchItem)]
pub fn search_item(Props { name, second }: &Props) -> Html{

    html!{
        <li class="search_item">
            {get_random_svg(true, "search_icon", "16")}
            <p>
                {name}
                <span>{second}</span>
            </p>
        </li>
    }
}