use crate::firewall::{
    FirewallHelperClient, FirewallHelperRequest, FirewallInspection, FirewallInspector,
};
use crate::ipc::response::IpcResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectRuleRequest {
    pub rule_name: String,
    pub expected_port_range: String,
    pub expected_proto: String,
}

#[tauri::command]
pub async fn firewall_inspect(request: InspectRuleRequest) -> IpcResult<FirewallInspection> {
    let result = FirewallInspector::inspect_rule(
        &request.rule_name,
        &request.expected_port_range,
        &request.expected_proto,
    );
    IpcResult::ok(result)
}

#[tauri::command]
pub async fn firewall_apply(rules: Vec<FirewallHelperRequest>) -> IpcResult<()> {
    match FirewallHelperClient::apply_rules(&rules) {
        Ok(()) => IpcResult::ok(()),
        Err(e) => IpcResult::err(e),
    }
}
