"use client";

import { useState } from "react";

const groups = [
  {
    id: "browser",
    owner: "SURF",
    count: 7,
    color: "blue",
    children: ["prepare", "navigate", "interact"],
  },
  {
    id: "studio",
    owner: "SYNESTHESIA",
    count: 47,
    color: "pink",
    children: ["composition", "inserts", "midi", "playback", "recording"],
  },
  {
    id: "canvas",
    owner: "CANVAS",
    count: 12,
    color: "orange",
    children: ["recall", "place", "navigate"],
  },
];

export default function Home() {
  const [depth, setDepth] = useState(2);

  return (
    <main>
      <nav className="nav">
        <a className="wordmark" href="#top" aria-label="DCP home">
          DCP<span className="cursor">_</span>
        </a>
        <div className="navlinks">
          <a href="#specification">SPECIFICATION</a>
          <a href="#streaming">STREAMING</a>
          <a href="#implementers">IMPLEMENTERS</a>
          <a href="#listing">GET LISTED ↗</a>
        </div>
      </nav>

      <section className="hero" id="top">
        <div className="eyebrow">DECISION CATALOG PROTOCOL · DRAFT 0.1</div>
        <h1>
          KNOW WHAT A<br />
          SYSTEM CAN <span>DO</span>
          <br />
          <em>RIGHT NOW.</em>
        </h1>
        <div className="hero-bottom">
          <p>
            A live, revision-bound directory of legal decisions for machines
            and models. Providers own reality. RLCD chooses. Execution stays
            safe.
          </p>
          <a className="round-link" href="#catalog" aria-label="Explore the catalog">
            ↓
          </a>
        </div>
      </section>

      <section className="principles" id="protocol">
        <div className="section-tag">01 / THE CONTRACT</div>
        <div className="principle-grid">
          <article>
            <b>PROVIDERS OWN REALITY</b>
            <p>Surf knows the browser. Synesthesia knows the studio. Canvas knows the screen.</p>
          </article>
          <article>
            <b>MODELS CHOOSE, NEVER INVENT</b>
            <p>RLCD selects from finite actions observed and authorized by deterministic code.</p>
          </article>
          <article>
            <b>EVERY CHOICE HAS A REVISION</b>
            <p>State moves. Stale actions fail closed. Every applied decision returns a receipt.</p>
          </article>
        </div>
      </section>

      <section className="spec" id="specification">
        <div className="section-tag">02 / PROTOCOL SURFACE</div>
        <div className="spec-intro">
          <h2>A SMALL CONTRACT.<br/>NO HIDDEN AUTHORITY.</h2>
          <p>DCP describes what a provider can decide and safely do against an exact observation. It does not define an agent loop, grant models executable selectors, or move authority out of the provider.</p>
        </div>
        <div className="spec-table">
          <div className="spec-row spec-header"><span>OPERATION</span><span>REQUIREMENT</span><span>PURPOSE</span></div>
          <div className="spec-row"><code>GET /v1/decisions</code><strong>MUST</strong><p>Return a finite, state-derived catalog with provider and observation revisions.</p></div>
          <div className="spec-row"><code>POST /v1/decisions/plan</code><strong>EXTENSION</strong><p>Provider-specific local selection; intentionally outside the DCP 0.1 core contract.</p></div>
          <div className="spec-row"><code>POST /v1/decisions/execute</code><strong>SHOULD</strong><p>Validate the expected revision, apply the action, and return a typed receipt.</p></div>
          <div className="spec-row"><code>/.well-known/dcp</code><strong>SHOULD</strong><p>Advertise protocol versions, transports, endpoints, and provider identity.</p></div>
        </div>
        <div className="spec-links">
          <a href="https://github.com/FuturePresentLabs/dcp/blob/main/spec/v0.1/README.md">NORMATIVE DCP 0.1 ↗</a>
          <a href="https://github.com/FuturePresentLabs/dcp/tree/main/public/schemas/v0.1">JSON SCHEMAS ↗</a>
          <a href="https://github.com/FuturePresentLabs/dcp/tree/main/conformance/v0.1">CONFORMANCE SUITE ↗</a>
        </div>
      </section>

      <section className="explorer" id="catalog">
        <div className="explorer-head">
          <div>
        <div className="section-tag light">03 / LIVE CATALOG</div>
            <h2>DECISIONS<br />DIRECTORY</h2>
          </div>
          <div className="endpoint">
            <span className="method">GET</span>
            <code>/v1/decisions?depth={depth}</code>
            <div className="depth" aria-label="Catalog depth">
              {[1, 2].map((value) => (
                <button
                  key={value}
                  className={depth === value ? "active" : ""}
                  onClick={() => setDepth(value)}
                >
                  DEPTH {value}
                </button>
              ))}
            </div>
          </div>
        </div>

        <div className="catalog-meta">
          <span>REVISION</span><strong>8FD3:B21A</strong>
          <span>FRESH FOR</span><strong>842ms</strong>
          <span>ACTIONS</span><strong>66</strong>
        </div>

        <div className="tree">
          {groups.map((group, index) => (
            <article className={`tree-group ${group.color}`} key={group.id}>
              <div className="branch-num">0{index + 1}</div>
              <div className="branch-main">
                <div className="owner">{group.owner} OWNS THIS BRANCH</div>
                <h3>{group.id}</h3>
                <span className="action-count">{group.count} LEGAL ACTIONS</span>
                {depth === 2 && (
                  <div className="children">
                    {group.children.map((child) => (
                      <div key={child}>
                        <span className="node-dot">●</span>
                        <code>{group.id}.{child}</code>
                        <span className="arrow">↗</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </article>
          ))}
        </div>
      </section>

      <section className="streaming" id="streaming">
        <div className="section-tag">04 / SPECULATIVE STREAMING</div>
        <div className="stream-head">
          <h2>DECIDE WHILE<br/>THE WORDS ARRIVE.</h2>
          <p>Incremental operation is part of DCP, not an implementation trick. Each evidence revision extends or supersedes prior evidence. Voice can revise per word; typed, vision, and sensor inputs use the same append-only chain.</p>
        </div>
        <div className="utterance" aria-label="Example incremental utterance">
          {["OPEN", "A", "BROWSER", "AND", "GO", "TO", "FPL.DEV"].map((word, index) => (
            <div className={index < 3 ? "prepared" : index === 6 ? "committed" : "observed"} key={word}>
              <span>r{index + 1}</span><b>{word}</b>
              <small>{index === 2 ? "PREPARE browser.ensure" : index === 6 ? "COMMIT browser.navigate" : "OBSERVE"}</small>
            </div>
          ))}
        </div>
        <div className="lifecycle">
          <article><span>01</span><b>OBSERVE</b><p>Accept a monotonic evidence revision and fresh provider catalogs.</p></article>
          <article><span>02</span><b>SPECULATE</b><p>Append a hypothesis referencing its parent decision and exact evidence.</p></article>
          <article><span>03</span><b>PREPARE</b><p>Only actions declared idempotent, reversible, and safe-before-final may run.</p></article>
          <article><span>04</span><b>COMMIT / CANCEL</b><p>Final evidence commits one valid action; superseded preparation is explicitly cancelled.</p></article>
        </div>
        <div className="normative-grid">
          <div>
            <h3>THE DECISION CHAIN</h3>
            <p>State accumulates as events—not an opaque model memory. Every decision names its parent, evidence revision, catalog revisions, phase, and resulting receipts.</p>
            <ul>
              <li>Evidence revisions MUST increase monotonically.</li>
              <li>A new decision MUST name the evidence it supersedes.</li>
              <li>Catalog changes MUST invalidate selections from an older revision.</li>
              <li>Finality MUST be explicit; confidence alone never commits a decision.</li>
            </ul>
          </div>
          <pre><code>{`{
  "dcp_version": "0.1",
  "message_id": "msg_03",
  "session_id": "sess_91",
  "sequence": 3,
  "occurred_at": "2026-10-03T06:00:00Z",
  "type": "decision.proposed",
  "payload": {
    "decision_id": "dec_03",
    "parent_decision_id": "dec_02",
    "chain_id": "voice:neo:turn-91",
    "evidence_revision": 3,
    "catalogs": { "surf": "cat_104" },
    "selection": {
      "provider_id": "surf",
      "action_id": "browser.ensure",
      "arguments": {}
    }
  }
}`}</code></pre>
        </div>
      </section>

      <section className="examples" id="examples">
        <div className="section-tag light">05 / WIRE EXAMPLE</div>
        <div className="example-layout">
          <div><h2>DISCOVER<br/>WHAT IS LEGAL.</h2><p>Depth changes representation, never authority. A cutoff node includes the exact bounded action IDs beneath it.</p></div>
          <pre><code>{`GET /v1/decisions?depth=2

{
  "dcp_version": "0.1",
  "provider": {
    "id": "synesthesia",
    "name": "Synesthesia",
    "instance": "https://studio.example/dcp"
  },
  "catalog_revision": "cat_022782",
  "state_revision": "state_619",
  "observed_at": "2026-10-03T06:00:00Z",
  "state": {
    "transport": "stopped",
    "recording": false
  },
  "actions": [{
    "id": "midi.disconnect.launchkey.piano",
    "domain": "midi",
    "phases": ["commit"],
    "safety": {
      "idempotent": true,
      "reversible": true,
      "requires_final": true,
      "confirmation_required": false
    }
  }]
}`}</code></pre>
        </div>
      </section>

      <section className="implementers" id="implementers">
        <div className="section-tag">06 / IMPLEMENTATIONS</div>
        <div className="directory-head"><h2>WHO SPEAKS DCP?</h2><p>Implementations are listed by the decisions they own. A listing is evidence of an inspectable catalog—not a certification of the underlying system.</p></div>
        <div className="implementation-list">
          <a href="https://github.com/FuturePresentLabs/synesthesia"><span>EXPERIMENTAL PROVIDER · 0.1 MIGRATION</span><b>SYNESTHESIA</b><p>State-derived studio catalog covering composition, inserts, MIDI, playback, and recording.</p><em>47 ACTIONS ↗</em></a>
          <a href="https://github.com/FuturePresentLabs/surf"><span>PROVIDER · IN PROGRESS</span><b>SURF</b><p>Revision-bound semantic browser observations and execution receipts.</p><em>BROWSER ↗</em></a>
          <a href="https://github.com/FuturePresentLabs/JARVIS"><span>CLIENT / COMPOSER · IN PROGRESS</span><b>JARVIS</b><p>Streams utterance revisions, composes provider catalogs, and gates speculative execution.</p><em>VOICE ↗</em></a>
        </div>
      </section>

      <section className="listing" id="listing">
        <div className="section-tag">07 / DIRECTORY POLICY</div>
        <div className="listing-grid">
          <div><h2>GET<br/>LISTED.</h2><p>The directory is open to interoperable implementations. Listings are generated from a small manifest and verified against a live or fixture catalog.</p></div>
          <ol>
            <li><span>01</span><div><b>PUBLISH DISCOVERY</b><p>Serve <code>/.well-known/dcp</code> with provider identity, supported protocol versions, and catalog endpoint.</p></div></li>
            <li><span>02</span><div><b>PROVIDE A FIXTURE</b><p>Include one redacted catalog response and its deterministic revision test.</p></div></li>
            <li><span>03</span><div><b>DECLARE SEMANTICS</b><p>Mark actions observe, prepare, or commit, including idempotency and reversibility.</p></div></li>
            <li><span>04</span><div><b>OPEN A LISTING PR</b><p>Add a valid entry with exact version, profile, source, contact, fixture, conformance command, and verification status.</p></div></li>
          </ol>
        </div>
        <a className="listing-cta" href="https://github.com/FuturePresentLabs/dcp/issues/new">SUBMIT AN IMPLEMENTATION ↗</a>
      </section>

      <section className="flow" id="start">
        <div className="section-tag">08 / EXECUTION LOOP</div>
        <h2>OBSERVE → CHOOSE → APPLY → PROVE</h2>
        <div className="flow-grid">
          <div><span>01</span><b>CATALOG</b><p>Ask the provider what is legal against its current state.</p></div>
          <div><span>02</span><b>DECIDE</b><p>Give RLCD only the bounded options and live evidence.</p></div>
          <div><span>03</span><b>EXECUTE</b><p>Submit the exact selection with its expected revision.</p></div>
          <div><span>04</span><b>RECEIPT</b><p>Verify what changed—or learn precisely why it did not.</p></div>
        </div>
        <div className="cta-row">
          <div>
            <small>THE OPEN PROTOCOL FOR BOUNDED INTELLIGENCE</small>
            <strong>BUILD A PROVIDER.</strong>
          </div>
          <a href="https://github.com/FuturePresentLabs/dcp/blob/main/spec/v0.1/README.md">READ DCP 0.1 ↗</a>
        </div>
      </section>

      <footer>
        <span>DECISIONS.DIRECTORY</span>
        <span>DECISION CATALOG PROTOCOL</span>
        <span>FPL · 2026</span>
      </footer>
    </main>
  );
}
