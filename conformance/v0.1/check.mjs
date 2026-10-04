import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import Ajv2020 from "ajv/dist/2020.js";
import addFormats from "ajv-formats";

const here = path.dirname(fileURLToPath(import.meta.url));
export const root = path.resolve(here, "../..");

export function readJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

export function createValidator() {
  const ajv = new Ajv2020({ allErrors: true, strict: true, strictRequired: false });
  addFormats(ajv);
  for (const name of ["common", "discovery", "catalog", "execute-request", "receipt", "stream-message", "directory-entry"]) {
    ajv.addSchema(readJson(`public/schemas/v0.1/${name}.json`));
  }
  return ajv;
}

export function assertSchema(ajv, schemaId, value) {
  const validate = ajv.getSchema(`https://decisions.directory/schemas/v0.1/${schemaId}.json`);
  if (!validate) throw new Error(`unknown schema ${schemaId}`);
  if (!validate(value)) throw new Error(ajv.errorsText(validate.errors, { separator: "\n" }));
}

export function catalogErrors(catalog) {
  const errors = [];
  const ids = new Set();
  for (const action of catalog.actions) {
    if (ids.has(action.id)) errors.push(`duplicate action ${action.id}`);
    ids.add(action.id);
  }
  const visit = (node) => {
    for (const actionId of node.action_ids ?? []) {
      if (!ids.has(actionId)) errors.push(`tree references unknown action ${actionId}`);
    }
    for (const child of node.children ?? []) visit(child);
  };
  for (const node of catalog.tree ?? []) visit(node);
  return errors;
}

export function streamErrors(messages) {
  const errors = [];
  const sessions = new Map();
  const evidence = new Map();
  const terminalChains = new Set();
  const catalogs = new Map();
  const decisions = new Map();
  const terminal = new Set();

  for (const message of messages) {
    const expectedSequence = (sessions.get(message.session_id) ?? 0) + 1;
    if (message.sequence !== expectedSequence) errors.push(`session ${message.session_id} expected sequence ${expectedSequence}, got ${message.sequence}`);
    sessions.set(message.session_id, message.sequence);

    const payload = message.payload;
    if (message.type === "evidence.observed") {
      const previous = evidence.get(payload.chain_id);
      if (terminalChains.has(payload.chain_id)) errors.push(`evidence ${payload.chain_id} changed after terminal status`);
      if (!previous && payload.revision !== 1) errors.push(`evidence ${payload.chain_id} must start at revision 1`);
      if (previous && payload.revision !== previous.revision + 1) errors.push(`evidence ${payload.chain_id} revision is not monotonic`);
      if (previous && payload.supersedes_revision !== previous.revision) errors.push(`evidence ${payload.chain_id} does not supersede its immediate predecessor`);
      evidence.set(payload.chain_id, payload);
      if (["final", "retracted"].includes(payload.status)) terminalChains.add(payload.chain_id);
    }
    if (message.type === "catalog.observed") catalogs.set(payload.provider_id, payload.catalog_revision);
    if (message.type === "decision.proposed") {
      if (decisions.has(payload.decision_id)) errors.push(`duplicate decision ${payload.decision_id}`);
      const observed = evidence.get(payload.chain_id);
      if (!observed || observed.revision < payload.evidence_revision) errors.push(`decision ${payload.decision_id} references unknown evidence revision`);
      if (payload.parent_decision_id !== null) {
        const parent = decisions.get(payload.parent_decision_id);
        if (!parent) errors.push(`decision ${payload.decision_id} references unknown or future parent`);
        else if (parent.chain_id !== payload.chain_id) errors.push(`decision ${payload.decision_id} parent belongs to another evidence chain`);
      }
      for (const [providerId, revision] of Object.entries(payload.catalogs)) {
        if (catalogs.get(providerId) !== revision) errors.push(`decision ${payload.decision_id} references unobserved catalog ${providerId}@${revision}`);
      }
      decisions.set(payload.decision_id, payload);
    }
    if (["action.prepared", "decision.committed", "decision.cancelled"].includes(message.type)) {
      if (!decisions.has(payload.decision_id)) errors.push(`${message.type} references unknown decision ${payload.decision_id}`);
      if (terminal.has(payload.decision_id)) errors.push(`decision ${payload.decision_id} changed after terminal event`);
      if (["decision.committed", "decision.cancelled"].includes(message.type)) terminal.add(payload.decision_id);
    }
  }
  return errors;
}

export function executeErrors(catalog, request) {
  const errors = [];
  const action = catalog.actions.find((candidate) => candidate.id === request.action_id);
  if (!action) return [`action_not_found:${request.action_id}`];
  if (request.expected_catalog_revision !== catalog.catalog_revision) errors.push("stale_catalog");
  if (request.expected_state_revision !== catalog.state_revision) errors.push("stale_state");
  if (!action.availability.available) errors.push(`action_unavailable:${action.availability.reason_code}`);
  if (!action.phases.includes(request.phase)) errors.push("unsafe_phase");
  if (request.phase === "commit" && action.safety.requires_final && request.evidence.status !== "final") errors.push("finality_required");
  if (request.phase === "commit" && action.safety.confirmation_required && !request.confirmed) errors.push("confirmation_required");
  if (request.phase === "cancel" && !request.prepared_receipt_id) errors.push("invalid_request");
  return errors;
}

export function validateFixtureSet() {
  const ajv = createValidator();
  for (const name of ["discovery", "catalog", "execute-request", "receipt", "directory-entry"]) {
    assertSchema(ajv, name, readJson(`fixtures/v0.1/valid/${name}.json`));
  }
  const stream = readJson("fixtures/v0.1/valid/stream.json");
  for (const message of stream) assertSchema(ajv, "stream-message", message);
  const semantic = [...catalogErrors(readJson("fixtures/v0.1/valid/catalog.json")), ...streamErrors(stream)];
  if (semantic.length) throw new Error(semantic.join("\n"));
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  validateFixtureSet();
  console.log("DCP 0.1 fixtures conform");
}
