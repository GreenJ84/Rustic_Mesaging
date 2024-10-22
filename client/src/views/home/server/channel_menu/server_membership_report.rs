use gloo::net::http::Request;
use yew::prelude::*;
use serde::Deserialize;
use crate::comps::csv_download::DownloadCsvButton;
use crate::comps::modal::Modal;
use crate::contexts::server_context::TServerContext;
use crate::models::report::ServerMemberReportItem;
use crate::utils::api_requests::api_get;
use crate::utils::format_date;

#[function_component(ServerMembershipReport)]
pub fn report_table() -> Html {
    let server_ctx = use_context::<TServerContext>().unwrap();
    let report_data = use_state(|| vec![]);

    {
        let report_data = report_data.clone();
        let server_ctx = server_ctx.clone();
        use_effect_with(server_ctx.current_server.clone(), move |_| {
            let server_ctx = server_ctx.clone();

            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(fetched_data) = api_get::<Vec<ServerMemberReportItem>>(String::from(format!("reports/server/{}", server_ctx.current_server.id))).await {
                    report_data.set(fetched_data);
                }
            });
            || ()
        });
    }

    html! {
        <Modal
            modal_class="server_membership_modal"
            button_class="server_membership report"
            button_icon={html!{"Membership Report"}}
        >
            <DownloadCsvButton
                url_extension={format!("reports/server/{}/csv", server_ctx.current_server.id)}
            />
            <table>
                <thead>
                    <tr>
                        <th>{ "Username" }</th>
                        <th>{ "Email" }</th>
                        <th>{ "Authorization" }</th>
                        <th>{ "Creation date" }</th>
                        <th>{ "Membership date" }</th>
                    </tr>
                </thead>
                <tbody>
                    { for report_data.iter().map(|item| html! {
                        <tr>
                            <td>{ &item.member.username }</td>
                            <td>{ &item.member.email }</td>
                            <td>{ if item.member.is_admin {"Admin"} else {"Member"} }</td>
                            <td>{ format_date(&item.member.created_at) }</td>
                            <td>{ format_date(&item.timestamp) }</td>
                        </tr>
                    }) }
                </tbody>
            </table>
        </Modal>
    }
}
