# Decision Catalog Protocol

Decision Catalog Protocol (DCP) is a small interoperability contract for
publishing the finite decisions a system can legally make **right now**.
Providers derive catalogs from live state, decision systems such as RLCD select
bounded actions, and providers retain authority over validation and execution.

This repository contains the normative DCP specification, machine-readable
schemas, conformance suite, public reference site, wire examples, implementation
directory, and listing policy. The intended public homes are
[decisions.directory](https://decisions.directory) and
[dcp.fpl.dev](https://dcp.fpl.dev).

## Protocol surface

- `GET /v1/decisions` publishes a state-derived, revision-bound catalog.
- `POST /v1/decisions/plan` optionally performs provider-local selection.
- `POST /v1/decisions/execute` validates and applies an exact bounded action.
- `/.well-known/dcp` advertises provider identity, versions, and endpoints.

The current draft is [DCP 0.1](spec/v0.1/README.md). Its JSON Schema 2020-12
contracts live under [`public/schemas/v0.1`](public/schemas/v0.1/) with stable
`$id` values at `https://decisions.directory/schemas/v0.1/`. Version 0.1 defines
exact version negotiation; discovery and authorization metadata; catalog,
action, execute, receipt, error, and directory-entry contracts; and a
transport-neutral streaming event model with a WebSocket binding.

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
`npm run test:conformance` runs only the protocol schema and semantic fixture
suite.

## Status

DCP is currently a draft protocol. Implementations may experiment with it, but
must advertise their supported protocol version and reject stale catalog
revisions.

## License

No license has been selected yet. All rights reserved until one is added.
