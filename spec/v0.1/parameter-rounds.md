# Parameter rounds profile 0.1

Status: normative draft profile for DCP 0.1. MUST, MUST NOT and SHOULD are
requirements as defined in the core specification.

Profile identifier: `https://decisions.directory/profiles/parameter-rounds/0.1`.
This is model-neutral; GPC-1 is one possible bounded-numeric client.

## Discovery and transport

A supporting provider MUST advertise the following entry in catalog `state`:

```json
{
  "https://decisions.directory/profiles/parameter-rounds/0.1": {
    "endpoint": "/v1/decisions/parameters"
  }
}
```

The endpoint MUST accept POST with `DCP-Version: 0.1` and JSON matching
`parameter-request.json`, and return JSON matching `parameter-round.json` with
the same version header. Endpoint URIs resolve against the catalog URL. Clients
MUST apply their normal destination/credential policy, not blindly forward
credentials to a cross-origin endpoint. Unsupported profiles MUST be rejected.

This namespaced state entry is compatible with core 0.1. Providers MUST NOT
add an unnegotiated standard discovery field. A client that does not implement
this profile may continue using the canonical action input schema.

## Collection, not execution

Clients first select an advertised action and, where required, its target.
The request carries an opaque request ID, action ID, exact catalog/state pins,
and the partial `arguments` already selected. Providers MUST authenticate and
authorize the request, reject unavailable actions or stale revisions, and
validate supplied argument names, types and constraints. Missing required
arguments are permitted only during collection.

A successful response MUST echo the profile, request ID, action, revisions and
arguments unchanged. Bounds and candidates MUST be computed against those
arguments and that exact state. Clients MUST verify the echo before use.
This endpoint MUST NOT modify application state, create parameters or bindings,
reserve resources, prepare an action, or issue an execution receipt. Repeating
an identical request against the same revisions MUST return equivalent domains.

Fields name unresolved numeric properties of the action input schema. Names
MUST be unique; each field MUST specify units, a reference frame, finite ordered
inclusive bounds (`minimum < maximum`), candidates and whether a bounded
estimate is permitted. Candidate IDs MUST be unique within a field. Candidate
values MUST be finite and within bounds. Each field MUST have at least one
candidate or allow a bounded estimate. A provider MUST NOT silently invent
defaults when it cannot establish a legal domain. It MUST return a typed error
and request clarification through its application when necessary.

Sources are `datum`, `named_parameter` and `prompt_literal`. These describe
provider-validated numeric values, not executable expressions or permanent
bindings. Prompt literals MUST be derived from identified user evidence, not
treated as a source of safety limits. New reusable parameters and persistent
bindings require separately advertised explicit actions.

## Numeric inference and commit

A bounded-numeric model may score a support (for example 101 evenly spaced
values). Support mapping, linear versus logarithmic spacing and readout are
client policy; this profile does not impose model logits on providers. The
client supplies resolved numbers in the normal execute arguments. Distribution
sharpness MUST NOT be represented as calibrated confidence without validation.

The client MUST discard collected domains if either revision changes. Providers
MUST validate final arguments against the current input schema, authorization,
revisions, target, numeric bounds and joint constraints at execute time. A legal
diameter and a legal depth individually need not form a legal bore together.
Collection MUST NOT bypass finality, confirmation, idempotency or prepare/commit
requirements. No Cartesian product of numeric combinations is required.

## Errors and conformance

Failures MUST use the core typed error contract. Recommended codes are
`unsupported_profile`, `stale_catalog`, `stale_state`, `invalid_arguments`,
`unavailable_action` and `parameter_domain_unavailable`. Schema validation alone
does not check ordered bounds, uniqueness by field name/candidate ID, echo
equality or joint constraints; conforming implementations MUST check these
semantic invariants as well.

The schemas live in `public/schemas/v0.1/parameter-request.json` and
`parameter-round.json`. Rust types live in `dcp::parameters`. This profile
standardizes numeric follow-up rounds; categorical targets remain ordinary
bounded choices. It does not claim any deployed provider or trained capability.
