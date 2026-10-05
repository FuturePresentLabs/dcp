import assert from "node:assert/strict";
import { readFile, access } from "node:fs/promises";
import test from "node:test";

test("Fab export includes rendered content, metadata, assets and schemas", {
  skip: process.env.FAB_STATIC_EXPORT !== "1",
}, async () => {
  const root = new URL("../dist/client/", import.meta.url);
  const html = await readFile(new URL("index.html", root), "utf8");
  assert.match(html, /DCP — Decision Catalog Protocol/);
  assert.match(html, /SPECULATIVE STREAMING/);
  assert.match(html, /src="https:\/\/understory\.fpl\.dev\/widget\.js"/);
  assert.match(html, /data-site="dcp"/);
  assert.match(html, /https:\/\/dcp\.fpl\.dev\/og.png/);
  const assets = [...html.matchAll(/(?:src|href)="(\/[^"?#]+\.(?:css|js))(?:\?[^"]*)?"/g)];
  assert.ok(assets.length > 0, "export must link its JS/CSS assets");
  for (const match of assets) {
    await access(new URL(match[1].slice(1), root));
  }
  await access(new URL("favicon.svg", root));
  await access(new URL("schemas/v0.1/catalog.json", root));
});
