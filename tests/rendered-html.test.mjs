import assert from "node:assert/strict";
import test from "node:test";

async function render() {
  const workerUrl = new URL("../dist/server/index.js", import.meta.url);
  workerUrl.searchParams.set("test", `${process.pid}-${Date.now()}`);
  const { default: worker } = await import(workerUrl.href);
  return worker.fetch(
    new Request("http://localhost/", { headers: { accept: "text/html" } }),
    { ASSETS: { fetch: async () => new Response("Not found", { status: 404 }) } },
    { waitUntil() {}, passThroughOnException() {} },
  );
}

test("renders the DCP standards reference", async () => {
  const response = await render();
  assert.equal(response.status, 200);
  const html = await response.text();
  assert.match(html, /DCP — Decision Catalog Protocol/);
  assert.match(html, /GET \/v1\/decisions/);
  assert.match(html, /SPECULATIVE STREAMING/);
  assert.match(html, /THE DECISION CHAIN/);
  assert.match(html, /WHO SPEAKS DCP/);
  assert.match(html, /GET<br\/>LISTED/);
  assert.doesNotMatch(html, /codex-preview|Your site is taking shape/);
});

test("documents monotonic evidence and bounded preparation", async () => {
  const html = await (await render()).text();
  assert.match(html, /Evidence revisions MUST increase monotonically/);
  assert.match(html, /idempotent, reversible, and safe-before-final/);
  assert.match(html, /parent_decision/);
  assert.match(html, /catalogs/);
  assert.match(html, /COMMIT \/ CANCEL/);
});
