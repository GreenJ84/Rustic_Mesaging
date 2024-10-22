use gloo::net::http::Request;
use yew::prelude::*;
use serde::Deserialize;
use crate::comps::csv_download::DownloadCsvButton;
use crate::comps::modal::Modal;
use crate::models::report::ActivityReportItem;
use crate::utils::api_requests::{api_get, API_URL};
use crate::utils::format_date;

#[function_component(MemberActivityReport)]
pub fn report_table() -> Html {
    let report_data = use_state(|| vec![]);

    {
        let report_data = report_data.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(fetched_data) = api_get::<Vec<ActivityReportItem>>(String::from("reports/member")).await {
                    report_data.set(fetched_data);
                }
            });
            || ()
        });
    }

    html! {
        <Modal
            modal_class="member_activity_modal"
            button_class="member_activity report"
            button_icon={html!{"Activity Report"}}
        >
            <DownloadCsvButton url_extension={String::from("reports/member/csv")}/>
            <table>
                <thead>
                    <tr>
                        <th>{ "Activity Type" }</th>
                        <th>{ "Description" }</th>
                        <th>{ "Timestamp" }</th>
                    </tr>
                </thead>
                <tbody>
                    { for report_data.iter().map(|item| html! {
                        <tr>
                            <td>{ &item.entity }</td>
                            <td>{ &item.description }</td>
                            <td>{ format_date(&item.timestamp) }</td>
                        </tr>
                    }) }
                </tbody>
            </table>
        </Modal>
    }
}
