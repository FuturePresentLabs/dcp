# DCP 0.1 conformance

Run the repository conformance suite with:

```bash
npm run test:conformance
```

The suite validates JSON fixtures against the normative JSON Schema 2020-12
documents and then checks semantics that JSON Schema cannot express: unique
action IDs, valid tree references, ordered stream sequences, immediate
transcript supersession, finality, observed catalog evidence, decision ancestry,
and terminal decisions.

Implementations claiming a DCP profile SHOULD run these fixtures through their
own encoder and decoder. Directory submissions MUST include a redacted fixture,
the exact profile/version claim, and a reproducible conformance command.

