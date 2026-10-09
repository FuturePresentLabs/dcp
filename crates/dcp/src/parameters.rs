//! Standard opt-in parameter-rounds/0.1 profile. Not an execution message.
//! Collecting parameters never prepares or commits an action.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const PROFILE: &str = "https://decisions.directory/profiles/parameter-rounds/0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterEndpoint {
    pub endpoint: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterRequest {
    pub profile: String,
    pub request_id: String,
    pub action_id: String,
    pub expected_catalog_revision: String,
    pub expected_state_revision: String,
    pub arguments: BTreeMap<String, Value>,
}

impl ParameterRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.profile != PROFILE
            || self.request_id.is_empty()
            || self.action_id.is_empty()
            || self.expected_catalog_revision.is_empty()
            || self.expected_state_revision.is_empty()
        {
            return Err("invalid parameter request identity or revisions");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterRound {
    pub profile: String,
    pub request_id: String,
    pub action_id: String,
    pub expected_catalog_revision: String,
    pub expected_state_revision: String,
    pub arguments: BTreeMap<String, Value>,
    pub fields: Vec<NumericField>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericField {
    pub name: String,
    pub unit: String,
    pub reference_frame: String,
    pub minimum: f64,
    pub maximum: f64,
    /// Finite, provider-validated values; selecting one does not create a binding.
    pub candidates: Vec<NumericCandidate>,
    pub allow_bounded_estimate: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericCandidate {
    pub id: String,
    pub value: f64,
    pub source: NumericSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumericSource {
    Datum,
    NamedParameter,
    PromptLiteral,
}

impl ParameterRound {
    pub fn validate_for(&self, request: &ParameterRequest) -> Result<(), &'static str> {
        request.validate()?;
        self.validate()?;
        if self.profile != request.profile
            || self.request_id != request.request_id
            || self.action_id != request.action_id
            || self.expected_catalog_revision != request.expected_catalog_revision
            || self.expected_state_revision != request.expected_state_revision
            || self.arguments != request.arguments
        {
            return Err("parameter round does not match request");
        }
        if self
            .fields
            .iter()
            .any(|field| self.arguments.contains_key(&field.name))
        {
            return Err("parameter round repeats an already resolved argument");
        }
        Ok(())
    }
    /// Providers must additionally check authorization, current revisions and
    /// target-specific joint constraints at execution time.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.profile != PROFILE
            || self.request_id.is_empty()
            || self.action_id.is_empty()
            || self.expected_catalog_revision.is_empty()
            || self.expected_state_revision.is_empty()
            || self.fields.is_empty()
        {
            return Err("parameter round requires an action, revisions and fields");
        }
        let mut names = std::collections::BTreeSet::new();
        for field in &self.fields {
            if field.name.is_empty()
                || !names.insert(&field.name)
                || field.unit.is_empty()
                || field.reference_frame.is_empty()
                || !field.minimum.is_finite()
                || !field.maximum.is_finite()
                || field.minimum >= field.maximum
            {
                return Err("invalid numeric field or duplicate field name");
            }
            let mut ids = std::collections::BTreeSet::new();
            for candidate in &field.candidates {
                if candidate.id.is_empty()
                    || !ids.insert(&candidate.id)
                    || !candidate.value.is_finite()
                    || candidate.value < field.minimum
                    || candidate.value > field.maximum
                {
                    return Err("invalid, duplicate or out-of-range numeric candidate");
                }
            }
            if field.candidates.is_empty() && !field.allow_bounded_estimate {
                return Err("numeric field has no answer path");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn round() -> ParameterRound {
        ParameterRound {
            profile: PROFILE.into(),
            request_id: "round-1".into(),
            arguments: BTreeMap::from([("target".into(), serde_json::json!("body.1"))]),
            action_id: "cut.bore".into(),
            expected_catalog_revision: "catalog-1".into(),
            expected_state_revision: "state-1".into(),
            fields: vec![NumericField {
                name: "diameter".into(),
                unit: "mm".into(),
                reference_frame: "part".into(),
                minimum: 1.,
                maximum: 10.,
                allow_bounded_estimate: true,
                candidates: vec![NumericCandidate {
                    id: "datum.bore".into(),
                    value: 5.,
                    source: NumericSource::Datum,
                }],
            }],
        }
    }
    #[test]
    fn validates_sources_bounds_and_revision_pins() {
        let mut value = round();
        assert!(value.validate().is_ok());
        value.fields[0].candidates[0].value = 11.;
        assert!(value.validate().is_err());
        value = round();
        value.expected_state_revision.clear();
        assert!(value.validate().is_err());
        value = round();
        value.fields.push(value.fields[0].clone());
        assert!(value.validate().is_err());
        value = round();
        value.fields[0].maximum = f64::NAN;
        assert!(value.validate().is_err());
    }
    #[test]
    fn strict_round_trip() {
        let value = round();
        let mut wire = serde_json::to_value(&value).unwrap();
        assert_eq!(
            serde_json::from_value::<ParameterRound>(wire.clone()).unwrap(),
            value
        );
        wire["commit"] = serde_json::json!(true);
        assert!(serde_json::from_value::<ParameterRound>(wire).is_err());
    }
    #[test]
    fn target_and_revision_echo_cannot_drift() {
        let mut value = round();
        let request = ParameterRequest {
            profile: value.profile.clone(),
            request_id: value.request_id.clone(),
            action_id: value.action_id.clone(),
            expected_catalog_revision: value.expected_catalog_revision.clone(),
            expected_state_revision: value.expected_state_revision.clone(),
            arguments: value.arguments.clone(),
        };
        assert!(value.validate_for(&request).is_ok());
        value
            .arguments
            .insert("target".into(), serde_json::json!("body.2"));
        assert!(value.validate_for(&request).is_err());
        value = round();
        value.expected_state_revision = "new-state".into();
        assert!(value.validate_for(&request).is_err());
    }
    #[test]
    fn normative_profile_examples_decode_and_match() {
        let request: ParameterRequest = serde_json::from_str(include_str!(
            "../../../fixtures/v0.1/valid/parameter-request.json"
        ))
        .unwrap();
        let round: ParameterRound = serde_json::from_str(include_str!(
            "../../../fixtures/v0.1/valid/parameter-round.json"
        ))
        .unwrap();
        round.validate_for(&request).unwrap();
    }
}
