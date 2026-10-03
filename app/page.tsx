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
          <a href="#protocol">PROTOCOL</a>
          <a href="#catalog">EXPLORER</a>
          <a href="#start">START BUILDING ↗</a>
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

      <section className="explorer" id="catalog">
        <div className="explorer-head">
          <div>
            <div className="section-tag light">02 / LIVE CATALOG</div>
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
          <span>ACTIONS</span><strong>43</strong>
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

      <section className="flow" id="start">
        <div className="section-tag">03 / ONE FAST LOOP</div>
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
          <a href="https://decisioncatalogprotocol.com">READ THE DRAFT ↗</a>
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
