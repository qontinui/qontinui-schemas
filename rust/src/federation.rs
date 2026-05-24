use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FederationReport {
    pub report_id: String,
    pub tenant_id: Option<String>,
    pub device_id: String,
    pub session_id: String,
    pub account_name: String,
    pub pushed: u32,
    pub pulled: u32,
    pub unchanged: u32,
    pub failed: u32,
    pub failed_names: Option<Vec<String>>,
    pub reported_at: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecordFederationReportRequest {
    pub session_id: String,
    pub account_name: String,
    pub pushed: u32,
    pub pulled: u32,
    pub unchanged: u32,
    pub failed: u32,
    pub failed_names: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FederationReportListResponse {
    pub items: Vec<FederationReport>,
}
