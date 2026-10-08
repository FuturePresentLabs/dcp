//! Canonical wire types for Decision Catalog Protocol 0.1.
//!
//! The JSON Schemas in `public/schemas/v0.1` remain normative. These types are
//! deliberately transport-neutral and reject unknown standard fields.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod context;

pub const VERSION: &str = "0.1";
pub const SPEC: &str = "https://decisions.directory/spec/v0.1";
pub const SCHEMA_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub instance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoints {
    pub catalog: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execute: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Authorization {
    pub scheme: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Discovery {
    pub dcp: String,
    pub provider: Provider,
    pub versions: Vec<String>,
    pub endpoints: Endpoints,
    pub authorization: Vec<Authorization>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub dcp_version: String,
    pub provider: Provider,
    pub catalog_revision: String,
    pub state_revision: String,
    pub observed_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<BTreeMap<String, Value>>,
    pub actions: Vec<Action>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tree: Vec<TreeNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub id: String,
    pub domain: String,
    pub title: String,
    pub description: String,
    pub input_schema: Value,
    pub phases: Vec<Phase>,
    pub safety: Safety,
    pub availability: Availability,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Prepare,
    Commit,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Safety {
    pub idempotent: bool,
    pub reversible: bool,
    pub requires_final: bool,
    pub confirmation_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Availability {
    pub available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub action_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecuteRequest {
    pub dcp_version: String,
    pub request_id: String,
    pub decision_id: String,
    pub action_id: String,
    pub arguments: BTreeMap<String, Value>,
    pub phase: Phase,
    pub expected_catalog_revision: String,
    pub expected_state_revision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepared_receipt_id: Option<String>,
    pub confirmed: bool,
    pub evidence: Evidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub chain_id: String,
    pub revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes_revision: Option<u64>,
    pub kind: String,
    pub status: EvidenceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    Partial,
    Final,
    Retracted,
}

impl Evidence {
    #[must_use]
    pub fn is_final(&self) -> bool {
        self.status == EvidenceStatus::Final
    }

    pub fn validate_revision(&self) -> Result<(), &'static str> {
        match (self.revision, self.supersedes_revision) {
            (0, _) => Err("evidence revision must start at one"),
            (1, None) => Ok(()),
            (1, Some(_)) => Err("evidence revision one must not supersede another revision"),
            (revision, Some(previous)) if previous + 1 == revision => Ok(()),
            (_, Some(_)) => Err("evidence must supersede the immediately preceding revision"),
            (_, None) => Err("evidence revisions after one must name supersedes_revision"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationOutcome {
    Confirmed,
    Replaced,
    Cancelled,
    Compensated,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub dcp_version: String,
    pub receipt_id: String,
    pub request_id: String,
    pub provider_id: String,
    pub action_id: String,
    pub phase: Phase,
    pub outcome: Outcome,
    pub expected_catalog_revision: String,
    pub expected_state_revision: String,
    pub observed_catalog_revision: String,
    pub observed_state_revision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resulting_catalog_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resulting_state_revision: Option<String>,
    pub observed_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<Problem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Prepared,
    Applied,
    Cancelled,
    Noop,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Problem {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<BTreeMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidRequest,
    UnsupportedVersion,
    Unauthorized,
    Forbidden,
    ActionNotFound,
    ActionUnavailable,
    StaleCatalog,
    StaleState,
    StaleEvidence,
    FinalityRequired,
    ConfirmationRequired,
    UnsafePhase,
    Conflict,
    CancellationFailed,
    InternalError,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/v0.1/valid")
                .join(name),
        )
        .unwrap()
    }

    #[test]
    fn normative_examples_decode_and_round_trip() {
        let discovery: Discovery = serde_json::from_str(&fixture("discovery.json")).unwrap();
        let catalog: Catalog = serde_json::from_str(&fixture("catalog.json")).unwrap();
        let request: ExecuteRequest =
            serde_json::from_str(&fixture("execute-request.json")).unwrap();
        let receipt: Receipt = serde_json::from_str(&fixture("receipt.json")).unwrap();
        let rejected: Receipt = serde_json::from_str(&fixture("receipt-stale-state.json")).unwrap();

        for value in [
            serde_json::to_value(discovery).unwrap(),
            serde_json::to_value(catalog).unwrap(),
            serde_json::to_value(request).unwrap(),
            serde_json::to_value(receipt).unwrap(),
            serde_json::to_value(rejected).unwrap(),
        ] {
            assert!(value.is_object());
        }
    }

    #[test]
    fn evidence_revisions_are_contiguous_and_terminal_status_is_explicit() {
        let first = Evidence {
            chain_id: "typed:neo:7".into(),
            revision: 1,
            supersedes_revision: None,
            kind: "text.command".into(),
            status: EvidenceStatus::Partial,
        };
        assert_eq!(first.validate_revision(), Ok(()));
        let second = Evidence {
            revision: 2,
            supersedes_revision: Some(1),
            status: EvidenceStatus::Final,
            ..first
        };
        assert_eq!(second.validate_revision(), Ok(()));
        assert!(second.is_final());
        let stale = Evidence {
            revision: 4,
            supersedes_revision: Some(1),
            ..second
        };
        assert!(stale.validate_revision().is_err());
    }
}
