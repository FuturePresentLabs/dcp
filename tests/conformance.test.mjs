import assert from "node:assert/strict";
import test from "node:test";
import { catalogErrors, createValidator, executeErrors, readJson, streamErrors, parameterRoundErrors } from "../conformance/v0.1/check.mjs";

const ajv = createValidator();

test("parameter rounds are strict, bounded and bound to the selected target", () => {
  const request = readJson("fixtures/v0.1/valid/parameter-request.json");
  const round = readJson("fixtures/v0.1/valid/parameter-round.json");
  assert.equal(validates("parameter-request", "fixtures/v0.1/valid/parameter-request.json"), true);
  assert.equal(validates("parameter-round", "fixtures/v0.1/valid/parameter-round.json"), true);
  assert.deepEqual(parameterRoundErrors(request, round), []);
  for (const mutate of [
    (r) => { r.arguments.target = "body.2"; },
    (r) => { r.expected_state_revision = "stale"; },
    (r) => { r.fields[0].maximum = 0; },
    (r) => { r.fields[0].candidates[0].value = 11; },
    (r) => { r.fields.push(structuredClone(r.fields[0])); },
    (r) => { r.fields[0].allow_bounded_estimate = false; r.fields[0].candidates = []; },
  ]) {
    const bad = structuredClone(round); mutate(bad);
    assert.ok(parameterRoundErrors(request, bad).length > 0);
  }
  const validate = ajv.getSchema("https://decisions.directory/schemas/v0.1/parameter-round.json");
  assert.equal(validate({...round, commit: true}), false);
});

function validates(schema, fixture) {
  const validate = ajv.getSchema(`https://decisions.directory/schemas/v0.1/${schema}.json`);
  return validate(readJson(fixture));
}

test("all normative valid fixtures pass their schemas", () => {
  for (const name of ["discovery", "catalog", "execute-request", "receipt", "directory-entry"]) {
    assert.equal(validates(name, `fixtures/v0.1/valid/${name}.json`), true, name);
  }
  const stream = readJson("fixtures/v0.1/valid/stream.json");
  const validate = ajv.getSchema("https://decisions.directory/schemas/v0.1/stream-message.json");
  for (const message of stream) assert.equal(validate(message), true, JSON.stringify(validate.errors));
});

test("invalid structural fixtures fail closed", () => {
  assert.equal(validates("catalog", "fixtures/v0.1/invalid/catalog-missing-safety.json"), false);
  assert.equal(validates("discovery", "fixtures/v0.1/invalid/discovery-credential.json"), false);
  assert.equal(validates("receipt", "fixtures/v0.1/invalid/receipt-rejected-without-error.json"), false);
});

test("valid catalogs and streams satisfy semantic invariants", () => {
  assert.deepEqual(catalogErrors(readJson("fixtures/v0.1/valid/catalog.json")), []);
  assert.deepEqual(streamErrors(readJson("fixtures/v0.1/valid/stream.json")), []);
});

test("sequence gaps are rejected even when individual messages are valid", () => {
  const stream = readJson("fixtures/v0.1/invalid/stream-gap.json");
  const validate = ajv.getSchema("https://decisions.directory/schemas/v0.1/stream-message.json");
  for (const message of stream) assert.equal(validate(message), true);
  assert.match(streamErrors(stream).join("\n"), /expected sequence 2, got 3/);
});

test("action phases are authoritative and stale state fails closed", () => {
  const catalog = readJson("fixtures/v0.1/valid/catalog.json");
  const commit = readJson("fixtures/v0.1/valid/execute-hue-scene.json");
  const prepare = readJson("fixtures/v0.1/invalid/execute-prepare-commit-only.json");
  const stale = readJson("fixtures/v0.1/invalid/execute-stale-state.json");
  const validate = ajv.getSchema("https://decisions.directory/schemas/v0.1/execute-request.json");
  assert.equal(validate(commit), true);
  assert.equal(validate(prepare), true);
  assert.equal(validate(stale), true);
  assert.deepEqual(executeErrors(catalog, commit), []);
  assert.ok(executeErrors(catalog, prepare).includes("unsafe_phase"));
  assert.ok(executeErrors(catalog, stale).includes("stale_state"));
});

test("stale-state rejection is a typed valid receipt", () => {
  assert.equal(validates("receipt", "fixtures/v0.1/valid/receipt-stale-state.json"), true);
});
