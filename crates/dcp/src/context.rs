//! Optional typed provider context under catalog.state["decision_context"].
//! Transport metadata only: no artifact fetching, model support or authority.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const SCHEMA: &str = "dcp.decision-context.v1";
pub const STATE_KEY: &str = "decision_context";

impl crate::Catalog {
    /// Read optional context against the canonical catalog state revision.
    pub fn decision_context(&self) -> Result<Option<DecisionContext>, String> {
        match &self.state {
            Some(state) => DecisionContext::from_state(state, &self.state_revision),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionContext {
    pub schema: String,
    pub source_revision: String,
    pub inputs: Vec<ContextInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    Structured,
    Geometry,
    Text,
    Image,
    Pdf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextInput {
    pub id: String,
    pub kind: InputKind,
    pub schema: String,
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<String>,
    pub source: ContextSource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ContextSource {
    Inline {
        value: Value,
    },
    Artifact {
        uri: String,
        sha256: String,
        media_type: String,
        byte_length: u64,
    },
}

impl DecisionContext {
    /// Validate metadata before model adaptation. Fetching requires separate
    /// provider authorization, URI allowlisting and byte/digest verification.
    pub fn validate(&self, expected_source_revision: &str) -> Result<(), String> {
        if self.schema != SCHEMA
            || expected_source_revision.is_empty()
            || self.source_revision != expected_source_revision
        {
            return Err("unsupported or stale decision context".into());
        }
        if self.inputs.is_empty() || self.inputs.len() > 16 {
            return Err("decision context requires 1..16 inputs".into());
        }
        let mut ids = BTreeSet::new();
        let mut inline_bytes = 0usize;
        for input in &self.inputs {
            if input.id.is_empty()
                || input.id.len() > 128
                || !ids.insert(&input.id)
                || input.schema.is_empty()
                || input.schema.len() > 256
            {
                return Err("context input ID/schema missing, duplicate or oversized".into());
            }
            for value in [&input.units, &input.frame].into_iter().flatten() {
                if value.is_empty() || value.len() > 128 {
                    return Err("invalid units/frame".into());
                }
            }
            if input.kind == InputKind::Geometry && (input.units.is_none() || input.frame.is_none())
            {
                return Err("geometry context requires units and frame".into());
            }
            match &input.source {
                ContextSource::Inline { value } => {
                    if matches!(input.kind, InputKind::Image | InputKind::Pdf) {
                        return Err("image/PDF context requires a pinned artifact".into());
                    }
                    if value.is_null() {
                        return Err("null context input".into());
                    }
                    inline_bytes = inline_bytes
                        .checked_add(serde_json::to_vec(value).map_err(|e| e.to_string())?.len())
                        .ok_or("context size overflow")?;
                    if inline_bytes > 256 * 1024 {
                        return Err("inline context exceeds 256KiB".into());
                    }
                }
                ContextSource::Artifact {
                    uri,
                    sha256,
                    media_type,
                    byte_length,
                } => {
                    // These are descriptors, not permission to dereference URLs.
                    if uri.len() > 2048
                        || uri.chars().any(char::is_whitespace)
                        || !(uri.starts_with("https://") && uri.len() > 8
                            || uri.starts_with("urn:") && uri.len() > 4)
                        || sha256.len() != 64
                        || !sha256
                            .bytes()
                            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                        || *byte_length == 0
                        || media_type.is_empty()
                        || media_type.len() > 128
                    {
                        return Err("invalid pinned context artifact".into());
                    }
                    if input.kind == InputKind::Pdf && media_type != "application/pdf"
                        || input.kind == InputKind::Image
                            && !matches!(
                                media_type.as_str(),
                                "image/png" | "image/jpeg" | "image/webp"
                            )
                    {
                        return Err("context modality/media type mismatch".into());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn from_state(
        state: &BTreeMap<String, Value>,
        source_revision: &str,
    ) -> Result<Option<Self>, String> {
        let Some(value) = state.get(STATE_KEY) else {
            return Ok(None);
        };
        let context: Self = serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
        context.validate(source_revision)?;
        Ok(Some(context))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> DecisionContext {
        DecisionContext {
            schema: SCHEMA.into(),
            source_revision: "r1".into(),
            inputs: vec![ContextInput {
                id: "faces".into(),
                kind: InputKind::Geometry,
                schema: "transmog.decision-geometry.v1".into(),
                required: true,
                units: Some("mm".into()),
                frame: Some("part-mm".into()),
                source: ContextSource::Inline {
                    value: serde_json::json!({"faces":[]}),
                },
            }],
        }
    }
    #[test]
    fn optional_context_is_revision_bound_and_strict() {
        let context = fixture();
        assert!(context.validate("r1").is_ok());
        assert!(context.validate("r2").is_err());
        assert_eq!(
            DecisionContext::from_state(&BTreeMap::new(), "r1").unwrap(),
            None
        );
        let mut value = serde_json::to_value(&context).unwrap();
        value["inputs"][0]["teacher_answer"] = serde_json::json!("face-1");
        assert!(serde_json::from_value::<DecisionContext>(value).is_err());
        let mut duplicate = context.clone();
        duplicate.inputs.push(context.inputs[0].clone());
        assert!(duplicate.validate("r1").is_err());
        let mut missing = context;
        missing.inputs[0].frame = None;
        assert!(missing.validate("r1").is_err());
    }
    #[test]
    fn pdf_and_image_require_pinned_artifacts_without_automatic_fetching() {
        let mut context = fixture();
        context.inputs[0].kind = InputKind::Pdf;
        assert!(context.validate("r1").is_err());
        context.inputs[0].source = ContextSource::Artifact {
            uri: "urn:provider:drawing".into(),
            sha256: "a".repeat(64),
            media_type: "application/pdf".into(),
            byte_length: 1024,
        };
        assert!(context.validate("r1").is_ok());
        if let ContextSource::Artifact { uri, .. } = &mut context.inputs[0].source {
            *uri = "file:///secret".into();
        }
        assert!(context.validate("r1").is_err());
    }
}
