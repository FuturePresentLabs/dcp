use std::{
    collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use dcp::{
    Action, Availability, Catalog, Discovery, Endpoints, ErrorCode, ExecuteRequest, Outcome, Phase,
    Problem, Provider, Receipt, Safety, TreeNode,
};
use serde_json::{json, Value};

const PROVIDER_ID: &str = "macos.local";

#[derive(Clone)]
struct AppState {
    provider: Provider,
}

#[derive(Debug, Clone)]
struct InstalledApp {
    id: String,
    name: String,
}

#[tokio::main]
async fn main() {
    let address = std::env::var("DCP_MACOS_BIND").unwrap_or_else(|_| "127.0.0.1:18841".into());
    let state = Arc::new(AppState {
        provider: Provider {
            id: PROVIDER_ID.into(),
            name: "macOS Local Control".into(),
            instance: format!("http://{address}"),
        },
    });
    let app = Router::new()
        .route("/.well-known/dcp", get(discovery))
        .route("/v1/decisions", get(catalog))
        .route("/v1/decisions/execute", post(execute))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("bind DCP macOS provider");
    println!("dcp-macos listening on {address}");
    axum::serve(listener, app)
        .await
        .expect("serve DCP macOS provider");
}

async fn discovery(State(state): State<Arc<AppState>>) -> Json<Discovery> {
    Json(Discovery {
        dcp: dcp::SPEC.into(),
        provider: state.provider.clone(),
        versions: vec![dcp::VERSION.into()],
        endpoints: Endpoints {
            catalog: "/v1/decisions".into(),
            execute: Some("/v1/decisions/execute".into()),
            stream: None,
        },
        authorization: vec![],
    })
}

async fn catalog(State(state): State<Arc<AppState>>) -> Json<Catalog> {
    Json(build_catalog(&state.provider))
}

async fn execute(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ExecuteRequest>,
) -> Json<Receipt> {
    let catalog = build_catalog(&state.provider);
    let result = validate_and_apply(&catalog, &request);
    Json(receipt(&catalog, &request, result))
}

fn build_catalog(provider: &Provider) -> Catalog {
    let apps = installed_apps();
    let revision = revision(&apps);
    let app_ids = apps
        .iter()
        .map(|app| Value::String(app.id.clone()))
        .collect::<Vec<_>>();
    let actions = vec![
        Action {
            id: "macos.app.open".into(),
            domain: "applications".into(),
            title: "Open an installed application".into(),
            description: "Activate an application currently installed on this Mac.".into(),
            input_schema: json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["app_id"],
                "properties": { "app_id": { "type": "string", "enum": app_ids } }
            }),
            phases: vec![Phase::Commit],
            safety: Safety {
                idempotent: true,
                reversible: false,
                requires_final: false,
                confirmation_required: false,
            },
            availability: Availability {
                available: !apps.is_empty(),
                reason_code: apps.is_empty().then(|| "no_installed_apps".into()),
                reason: None,
            },
        },
        Action {
            id: "macos.audio.volume.set".into(),
            domain: "audio".into(),
            title: "Set system output volume".into(),
            description: "Set the local Mac output volume from 0 through 100.".into(),
            input_schema: json!({
                "type": "object", "additionalProperties": false, "required": ["percent"],
                "properties": { "percent": { "type": "integer", "minimum": 0, "maximum": 100 } }
            }),
            phases: vec![Phase::Commit],
            safety: Safety {
                idempotent: true,
                reversible: true,
                requires_final: false,
                confirmation_required: false,
            },
            availability: Availability {
                available: true,
                reason_code: None,
                reason: None,
            },
        },
        Action {
            id: "macos.audio.mute.set".into(),
            domain: "audio".into(),
            title: "Set system output mute".into(),
            description: "Mute or unmute the local Mac output.".into(),
            input_schema: json!({
                "type": "object", "additionalProperties": false, "required": ["muted"],
                "properties": { "muted": { "type": "boolean" } }
            }),
            phases: vec![Phase::Commit],
            safety: Safety {
                idempotent: true,
                reversible: true,
                requires_final: false,
                confirmation_required: false,
            },
            availability: Availability {
                available: true,
                reason_code: None,
                reason: None,
            },
        },
    ];
    Catalog {
        dcp_version: dcp::VERSION.into(),
        provider: provider.clone(),
        catalog_revision: format!("cat:macos:{revision:016x}"),
        state_revision: format!("state:macos:{revision:016x}"),
        observed_at: Utc::now().to_rfc3339(),
        expires_at: None,
        state: Some(BTreeMap::from([(
            "installed_app_count".into(),
            json!(apps.len()),
        )])),
        actions,
        tree: vec![
            TreeNode {
                id: "applications".into(),
                label: "Applications".into(),
                action_ids: vec!["macos.app.open".into()],
                children: vec![],
            },
            TreeNode {
                id: "audio".into(),
                label: "Audio".into(),
                action_ids: vec![
                    "macos.audio.volume.set".into(),
                    "macos.audio.mute.set".into(),
                ],
                children: vec![],
            },
        ],
    }
}

fn validate_and_apply(catalog: &Catalog, request: &ExecuteRequest) -> Result<Outcome, Problem> {
    if request.dcp_version != dcp::VERSION {
        return Err(problem(
            ErrorCode::UnsupportedVersion,
            "unsupported DCP version",
        ));
    }
    request
        .evidence
        .validate_revision()
        .map_err(|message| problem(ErrorCode::InvalidRequest, message))?;
    if request.expected_catalog_revision != catalog.catalog_revision {
        return Err(problem(ErrorCode::StaleCatalog, "catalog revision changed"));
    }
    if request.expected_state_revision != catalog.state_revision {
        return Err(problem(ErrorCode::StaleState, "state revision changed"));
    }
    if request.phase != Phase::Commit {
        return Err(problem(
            ErrorCode::UnsafePhase,
            "macOS actions support commit only",
        ));
    }
    match request.action_id.as_str() {
        "macos.app.open" => {
            let id = string_argument(request, "app_id")?;
            let app = installed_apps()
                .into_iter()
                .find(|app| app.id == id)
                .ok_or_else(|| {
                    problem(
                        ErrorCode::ActionUnavailable,
                        "application is no longer installed",
                    )
                })?;
            run(Command::new("/usr/bin/open").args(["-a", &app.name]))?;
        }
        "macos.audio.volume.set" => {
            let value = integer_argument(request, "percent")?;
            if !(0..=100).contains(&value) {
                return Err(problem(
                    ErrorCode::InvalidRequest,
                    "percent must be 0 through 100",
                ));
            }
            run(Command::new("/usr/bin/osascript")
                .args(["-e", &format!("set volume output volume {value}")]))?;
        }
        "macos.audio.mute.set" => {
            let muted = request
                .arguments
                .get("muted")
                .and_then(Value::as_bool)
                .ok_or_else(|| problem(ErrorCode::InvalidRequest, "muted must be boolean"))?;
            run(Command::new("/usr/bin/osascript")
                .args(["-e", &format!("set volume output muted {muted}")]))?;
        }
        _ => {
            return Err(problem(
                ErrorCode::ActionNotFound,
                "action is not in the live catalog",
            ))
        }
    }
    Ok(Outcome::Applied)
}

fn receipt(
    catalog: &Catalog,
    request: &ExecuteRequest,
    result: Result<Outcome, Problem>,
) -> Receipt {
    let (outcome, error) = match result {
        Ok(outcome) => (outcome, None),
        Err(error) => (Outcome::Rejected, Some(error)),
    };
    Receipt {
        dcp_version: dcp::VERSION.into(),
        receipt_id: format!("receipt:macos:{}", request.request_id),
        request_id: request.request_id.clone(),
        provider_id: PROVIDER_ID.into(),
        action_id: request.action_id.clone(),
        phase: request.phase.clone(),
        outcome,
        expected_catalog_revision: request.expected_catalog_revision.clone(),
        expected_state_revision: request.expected_state_revision.clone(),
        observed_catalog_revision: catalog.catalog_revision.clone(),
        observed_state_revision: catalog.state_revision.clone(),
        resulting_catalog_revision: None,
        resulting_state_revision: None,
        observed_at: Utc::now().to_rfc3339(),
        error,
    }
}

fn installed_apps() -> Vec<InstalledApp> {
    let mut names = BTreeSet::new();
    for root in application_roots() {
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) == Some("app") {
                    if let Some(name) = path.file_stem().and_then(|value| value.to_str()) {
                        names.insert(name.to_owned());
                    }
                }
            }
        }
    }
    names
        .into_iter()
        .map(|name| InstalledApp {
            id: slug(&name),
            name,
        })
        .collect()
}

fn application_roots() -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        roots.push(Path::new(&home).join("Applications"));
    }
    roots
}

fn slug(name: &str) -> String {
    let value = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    value
        .trim_matches('-')
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn revision(apps: &[InstalledApp]) -> u64 {
    let mut h = DefaultHasher::new();
    for app in apps {
        app.id.hash(&mut h);
        app.name.hash(&mut h);
    }
    h.finish()
}
fn string_argument<'a>(request: &'a ExecuteRequest, name: &str) -> Result<&'a str, Problem> {
    request
        .arguments
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| problem(ErrorCode::InvalidRequest, "missing string argument"))
}
fn integer_argument(request: &ExecuteRequest, name: &str) -> Result<i64, Problem> {
    request
        .arguments
        .get(name)
        .and_then(Value::as_i64)
        .ok_or_else(|| problem(ErrorCode::InvalidRequest, "missing integer argument"))
}
fn run(command: &mut Command) -> Result<(), Problem> {
    let status = command.status().map_err(|_| {
        problem(
            ErrorCode::InternalError,
            "local control process failed to start",
        )
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(problem(
            ErrorCode::InternalError,
            "local control process failed",
        ))
    }
}
fn problem(code: ErrorCode, message: impl Into<String>) -> Problem {
    Problem {
        code,
        message: message.into(),
        retryable: false,
        details: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_finite_and_state_derived() {
        let provider = Provider {
            id: PROVIDER_ID.into(),
            name: "test".into(),
            instance: "http://127.0.0.1:18841".into(),
        };
        let catalog = build_catalog(&provider);
        assert!(catalog.actions.iter().all(
            |action| action.availability.available || action.availability.reason_code.is_some()
        ));
        assert!(catalog
            .actions
            .iter()
            .any(|action| action.id == "macos.app.open"));
    }

    #[test]
    fn app_ids_are_bounded_safe_selectors() {
        assert_eq!(slug("Visual Studio Code"), "visual-studio-code");
        assert_eq!(slug("A/B.app"), "a-b-app");
    }
}
