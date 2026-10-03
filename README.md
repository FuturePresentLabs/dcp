# Decision Catalog Protocol

Decision Catalog Protocol (DCP) is a small interoperability contract for
publishing the finite decisions a system can legally make **right now**.
Providers derive catalogs from live state, decision systems such as RLCD select
bounded actions, and providers retain authority over validation and execution.

This repository contains the public DCP specification site, wire examples, the
implementation directory, and the listing policy. The intended public homes are
[decisions.directory](https://decisions.directory) and
[dcp.fpl.dev](https://dcp.fpl.dev).

## Protocol surface

- `GET /v1/decisions` publishes a state-derived, revision-bound catalog.
- `POST /v1/decisions/plan` optionally performs provider-local selection.
- `POST /v1/decisions/execute` validates and applies an exact bounded action.
- `/.well-known/dcp` advertises provider identity, versions, and endpoints.

DCP also defines speculative, per-word operation as an append-only decision
chain. Transcript revisions accumulate evidence; safe actions may be prepared
early; explicit finality commits one valid selection; superseded preparations
are cancelled. Confidence alone never grants execution authority.

## Implementations and directory listings

The site lists DCP providers and clients with links to their source. To request
a listing, publish `/.well-known/dcp`, provide a deterministic catalog fixture,
declare each action's execution semantics, and open a listing issue or pull
request in this repository.

## Local development

Requires Node.js 22.13 or newer.

```bash
npm install
npm test
npm run start
```

The production preview is available at <http://localhost:3000> by default.

## Status

DCP is currently a draft protocol. Implementations may experiment with it, but
must advertise their supported protocol version and reject stale catalog
revisions.

## License

No license has been selected yet. All rights reserved until one is added.
