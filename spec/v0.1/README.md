# Decision Catalog Protocol 0.1

Status: **Draft**. This document uses the key words MUST, MUST NOT, REQUIRED,
SHOULD, SHOULD NOT, and MAY as described by RFC 2119 and RFC 8174.

## 1. Scope

DCP lets a provider publish the finite actions legal against its current state,
lets a bounded decision system select one action, and lets the provider validate
and execute that selection. The provider remains authoritative. A decision
system, including RLCD, MUST output only an `action_id` and JSON arguments from
the advertised schema; it MUST NOT output provider-internal executable selectors
such as code, shell commands, URLs not accepted by the action schema, or object
handles.

The schemas in [`public/schemas/v0.1`](../../public/schemas/v0.1/) are normative. Examples and
fixtures are non-normative unless a conformance profile explicitly requires
them.

## 2. Version negotiation

A provider MUST advertise every exact supported version in discovery. An HTTP
client MUST send `DCP-Version` with one advertised value and the provider MUST
echo the selected value in `DCP-Version`. Implementations MUST NOT infer
compatibility between distinct version strings. A provider unable to select the
requested version MUST return HTTP 406 with `unsupported_version`.

Version `0.1` uses JSON Schema 2020-12. Extensions MUST use names containing a
reverse-DNS or URI namespace and MUST NOT change the meaning of standard fields.
Unknown optional extension fields MAY be ignored. Unknown standard fields MUST
NOT be treated as authority.

Provider-local planning endpoints, including `/v1/decisions/plan`, are outside
the DCP 0.1 core contract. They MAY be implemented as explicitly documented
extensions, but their output has no execution authority until it is represented
as a bounded DCP selection and validated by the provider.

## 3. Discovery and authorization

`GET /.well-known/dcp` MUST return a `discovery` document. It MUST identify the
provider, supported versions, catalog endpoint, and authorization schemes. It
MAY advertise execute and stream endpoints. Endpoint URIs MAY be relative to the
discovery URI.

Discovery MUST NOT contain credentials, bearer tokens, API keys, cookies,
private keys, or client secrets. Authorization metadata describes how a client
obtains or presents authority; it never grants authority. Providers MUST
authenticate and authorize execution independently of catalog visibility.

## 4. Catalogs and actions

`GET /v1/decisions` is the recommended HTTP catalog endpoint. A catalog MUST
bind its contents to an opaque `catalog_revision` and the observed provider
state to an opaque `state_revision`. Revisions are equality tokens; clients MUST
NOT assume lexical or numeric ordering. Providers SHOULD include `expires_at`
when freshness has a known bound.

The flat `actions` collection is canonical. A tree is an optional presentation
projection and MUST reference only action IDs in that same catalog. A `depth`
projection MUST NOT change action authority or revisions.

Every action MUST declare an input schema, allowed lifecycle phases,
availability, and all four safety properties:

- `idempotent`: replay with the same request ID has no additional effect.
- `reversible`: the provider has a bounded compensating or cancellation action.
- `requires_final`: commit requires evidence whose status is `final`.
- `confirmation_required`: commit requires explicit confirmation evidence.

A provider MUST reject a phase not listed by the action. Phase membership is
the authority for lifecycle support: clients MUST NOT infer prepare support from
`idempotent` or `reversible`. Preparation before finality is permitted only when
`prepare` is listed, `requires_final` is false, and provider policy permits it.
Clients SHOULD prepare only idempotent and reversible actions. An unavailable
action MUST include a machine-readable `reason_code`. Availability is advisory;
execute MUST validate again.

## 5. Lifecycle

Providers MAY implement the standard opt-in
[parameter rounds profile](parameter-rounds.md) to collect bounded numeric
arguments after action and target selection. Collection is read-only and does
not grant prepare or commit authority.

The lifecycle is `observe → speculate → prepare → commit | cancel`.

1. **Observe:** clients accumulate source evidence and catalog evidence without an
   effect.
2. **Speculate:** a decision system appends a proposed bounded selection. A
   proposal is not execution authority.
3. **Prepare:** the provider MAY establish a reversible intermediate state and
   returns a `prepared` receipt. Prepare MUST NOT claim the committed effect.
4. **Commit:** the provider validates current revisions, finality, confirmation,
   and authorization, then applies at most one effect for a request ID.
5. **Cancel:** a client explicitly cancels a preparation. Providers MUST make
   cancellation idempotent. A no-longer-present preparation returns `noop`;
   failure to restore a promised reversible state returns `cancellation_failed`.

Commit and cancel of a prepared operation MUST reference its preparation
receipt. A superseded proposal SHOULD cause cancellation of preparations that
are no longer reachable from the active decision chain.

## 6. Streaming and accumulated state

Core messages are transport-neutral and use the `stream-message` schema. Within
a session, `sequence` MUST increase by exactly one. Events are append-only and
MUST NOT mutate earlier events.

For one evidence chain, revisions MUST increase by exactly one. Revision one
MUST omit `supersedes_revision`; every later revision MUST name the previous
revision. `value` is the complete evidence value at that revision, not a delta.
`kind` identifies its provider-neutral semantics (for example
`text.transcript`, `text.command`, `presence.snapshot`, or `vision.scene`).
`status` is `partial`, `final`, or `retracted`; `final` and `retracted` are
terminal, so no later revision is valid. DCP does not define how a producer
tokenizes, captures, or derives evidence.

Each `decision.proposed` MUST reference exact evidence and catalog revisions.
Its `parent_decision_id` MUST refer to an earlier decision in the same evidence chain
or be null for the root. The parent relation MUST be acyclic. A new decision
does not erase its parent; it adds a new chain node.

`decision.committed` and `decision.cancelled` are terminal for that decision.
No later execution event may revive a terminal decision. Consumers rebuilding
state MUST fold events in sequence order and MUST fail closed on a gap,
duplicate, invalid parent, or unknown revision.

Final evidence MUST reconcile every prepared or proposed decision reachable
from the chain. A `decision.reconciled` event records `confirmed`, `replaced`,
`cancelled`, or `compensated`; replacement names the new decision and
compensation names the receipt when one exists. Reconciliation records history
and grants no execution authority by itself.

### WebSocket binding

A provider MAY advertise a `wss` stream endpoint. Clients MUST request the
`dcp.v0.1` WebSocket subprotocol. Each text frame MUST contain exactly one JSON
stream message. Binary frames are not defined. Delivery is ordered within a
connection. Reconnect begins a new `session_id`; replay is not implied unless a
future extension is explicitly negotiated.

## 7. Revision-bound execution

An execute request MUST carry the exact expected catalog and state revisions.
The provider MUST compare both immediately before applying an effect. A mismatch
MUST produce a rejected receipt with `stale_catalog` or `stale_state`; it MUST
NOT apply the requested effect. Clients MUST refresh the catalog and obtain a
new decision rather than silently substituting revisions.

Every accepted execute request MUST return a receipt, including `noop` and
rejection outcomes. Request IDs are idempotency keys within a provider-defined
retention window. Reuse with different content MUST return `conflict`.

## 8. Errors

Errors use the `problem` shape and one of these stable codes:

| Code | Meaning | Recommended HTTP |
| --- | --- | --- |
| `invalid_request` | Schema or semantic failure | 400 |
| `unsupported_version` | No exact version match | 406 |
| `unauthorized` | Authentication absent or invalid | 401 |
| `forbidden` | Identity lacks action authority | 403 |
| `action_not_found` | Action ID is unknown | 404 |
| `action_unavailable` | Action exists but is not currently legal | 409 |
| `stale_catalog` | Catalog revision differs | 409 |
| `stale_state` | State revision differs | 409 |
| `stale_evidence` | Evidence revision was superseded or retracted | 409 |
| `finality_required` | Commit lacks final evidence | 409 |
| `confirmation_required` | Commit lacks required confirmation | 409 |
| `unsafe_phase` | Action does not permit the phase | 409 |
| `conflict` | Idempotency key was reused incompatibly | 409 |
| `cancellation_failed` | Reversible preparation could not be restored | 500 |
| `internal_error` | Unclassified provider failure | 500 |

Error details MUST NOT disclose credentials or sensitive internal selectors.

## 9. Conformance profiles

- **Catalog Provider:** valid discovery and catalog documents; exact version and
  revision behavior; action schemas and safety declarations.
- **Executor:** Catalog Provider plus revision-bound execute, idempotency,
  receipts, authorization enforcement, lifecycle and error behavior.
- **Streaming:** valid stream messages plus monotonic sequence, evidence
  supersession, append-only decision-chain, terminality, and cancellation.
- **Directory Listed:** one or more named profiles, public source or documented
  implementation, valid redacted fixtures, conformance command and versioned
  compatibility claims.

A compatibility claim MUST name an exact DCP version and profile. “DCP
compatible” without both is not a conformant claim. Directory entries MUST use
the directory-entry schema and MUST distinguish `self-tested` from `verified`.

## Optional typed decision context

Providers MAY include `state.decision_context`, described by
`decision-context.json` and the Rust `dcp::context` module. This extension does
not alter action input schemas, execution authorization or receipts.
Its `source_revision` MUST equal the catalog's `state_revision`. Input IDs MUST
be unique. The context and its immutable artifact descriptors MUST participate
in catalog revision identity; changing an input requires a new catalog revision.

Inputs declare a kind (`structured`, `geometry`, `text`, `image`, `pdf`), a
provider-owned schema identifier and whether the input is required. Geometry
MUST specify units and a coordinate frame. The recipe owns feature extraction,
tokenization and embeddings; none of those are protocol fields. An advertised
input is not a claim that a model supports that modality. Consumers MUST reject
unsupported required inputs before inference; unsupported optional inputs MAY
be omitted and that omission SHOULD be reported by the inference frontend.

Structured/text/geometry inputs may be inline, with a total serialized inline
budget of 256KiB. Images and PDFs MUST use artifact descriptors with URI, SHA256,
media type and exact byte length. Inline null values are invalid. Consumers MUST
validate the source revision, uniqueness and size bounds in addition to JSON
Schema. An artifact URI is not permission to fetch: resolvers MUST enforce
provider authorization, URI/host allowlists, redirect restrictions and their own
size limits, then verify bytes, digest and media type before decoding. No local
filesystem or data URI is allowed. URNs require a provider-specific resolver.
PDF parsing and rasterization require separate bounded/sandboxed decoders;
document contents are untrusted observations, not instructions or authority.

This is optional context discovery, not model-generated context, a new tool-call
format, or manufacturing certification. Do not embed teacher answers, verdicts
or private credentials in context offered to a decision model.
