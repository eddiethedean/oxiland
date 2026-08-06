---
hide:
  - navigation
  - toc
title: Oxiland — embedded RDF for Rust and Python
description: Build typed, local RDF applications with SPARQL, persistent datasets, and streaming I/O in Rust or Python.
---

!!! info "Release status"

    This documentation tip describes **0.12.0** (the current published package
    version). Pin installs as `oxiland = "0.12.0"` / `pip install oxiland==0.12.0`,
    or use a git/path checkout for unreleased tip APIs.

<section class="hero" aria-labelledby="hero-title">
  <div class="hero__copy">
    <p class="hero__eyebrow">Oxiland</p>
    <h1 id="hero-title">Typed RDF, SPARQL, and local persistence—without a database server.</h1>
    <p class="hero__lead">
      Build linked-data applications with validated terms, named graphs, SPARQL
      1.1, streaming I/O, and an embedded durable store inside your process.
    </p>
    <div class="hero__actions">
      <a class="md-button md-button--primary" href="users/python/">Start with Python</a>
      <a class="md-button" href="users/rust/">Start with Rust</a>
    </div>
    <div class="hero__meta" aria-label="Project highlights">
      <span>Apache-2.0 OR MIT</span>
      <span>Rust 1.87+</span>
      <span>Python 3.10–3.14</span>
    </div>
  </div>
  <div class="hero__terminal" aria-label="Python quick start example">
    <div class="hero__terminal-bar" aria-hidden="true">
      <span></span><span></span><span></span>
      <strong>quick_start.py</strong>
    </div>

    <pre><code class="language-python">from oxiland import Model, load, query

graph = Model()
load(
    graph,
    '&lt;alice&gt; &lt;name&gt; "Alice" .',
    "turtle",
    base_iri="https://example.com/",
)

assert query(graph, "ASK { ?s ?p ?o }")</code></pre>
  </div>
</section>

<div class="trust-strip" role="list" aria-label="Core capabilities">
  <span role="listitem"><b aria-hidden="true">✓</b> Safe, typed APIs</span>
  <span role="listitem"><b aria-hidden="true">✓</b> Local persistence</span>
  <span role="listitem"><b aria-hidden="true">✓</b> SPARQL 1.1</span>
  <span role="listitem"><b aria-hidden="true">✓</b> Streaming RDF I/O</span>
</div>

!!! success "Suite-wide faster-than-Redland authorized"

    Tip closed the ADR-028 competitive-parity gate and the ADR-029 suite-wide
    faster-than-Redland gate (Linux, macOS, and Windows × three independent
    strict runs). See the [performance guide](users/performance.md) and the
    [0.13 report](reports/0.13.md) for claims policy and tables.

## Choose your path

<div class="path-grid">

<a class="path-card" href="users/python/">
  <span class="path-card__badge" aria-hidden="true">Py</span>
  <h3>Python</h3>
  <p>Go from <code>pip install</code> to a queried dataset in five minutes.</p>
  <span class="path-card__link">Open the Python guide →</span>
</a>

<a class="path-card" href="users/rust/">
  <span class="path-card__badge" aria-hidden="true">Rs</span>
  <h3>Rust</h3>
  <p>Use a compact, safe API for models, storage, queries, and RDF streams.</p>
  <span class="path-card__link">Open the Rust guide →</span>
</a>

<a class="path-card" href="users/cli/">
  <span class="path-card__badge" aria-hidden="true">$_</span>
  <h3>Command line</h3>
  <p>Import, inspect, query, and export local datasets from scripts or a shell.</p>
  <span class="path-card__link">Open the CLI guide →</span>
</a>

<a class="path-card" href="users/c-abi/">
  <span class="path-card__badge" aria-hidden="true">C</span>
  <h3>C ABI preview</h3>
  <p>Link a Redland-shaped source-compat preview against a frozen allowlist.</p>
  <span class="path-card__link">Open the C ABI guide →</span>
</a>

</div>

## One toolkit, the complete local workflow

<div class="feature-grid">

<div class="feature-card">
  <h3>Model RDF precisely</h3>
  <p>Validated IRIs, blank nodes, literals, triples, quads, default graphs, and named graphs give applications a clear data contract.</p>
</div>

<div class="feature-card">
  <h3>Own the data lifecycle</h3>
  <p>Choose an in-memory model or a durable local store with atomic transactions, explicit sync, read-only access, and portable N-Quads backups.</p>
</div>

<div class="feature-card">
  <h3>Query with SPARQL</h3>
  <p>Run ASK, SELECT, CONSTRUCT, DESCRIBE, and Update. Consume large result sets as lazy iterators instead of materializing them all at once.</p>
</div>

<div class="feature-card">
  <h3>Stream standard formats</h3>
  <p>Read and write Turtle, N-Triples, N-Quads, TriG, and RDF/XML with explicit syntax and import-failure semantics.</p>
</div>

</div>

## Ready for more than a demo

Oxiland is an embedded library, not a hosted database. Your application owns
the store path, permissions, process lifecycle, capacity, backups, and network
boundary. The production guides turn those responsibilities into an operating
model.

<div class="next-grid">

<div class="next-card">
  <strong>Deploy safely</strong>
  <p><a href="users/python-production/">Python operations</a> · <a href="users/rust-production/">Rust operations</a></p>
</div>

<div class="next-card">
  <strong>Look up an API</strong>
  <p><a href="users/python-api/">Python reference</a> · <a href="https://docs.rs/oxiland">Rust reference</a></p>
</div>

<div class="next-card">
  <strong>Solve a problem</strong>
  <p><a href="users/faq/">FAQ and troubleshooting</a> · <a href="support/">Support policy</a></p>
</div>

<div class="next-card">
  <strong>Evaluate the fit</strong>
  <p><a href="evaluators/positioning/">Positioning</a> · <a href="parity/">Compatibility evidence</a></p>
</div>

</div>

!!! info "Compatibility claims are evidence-scoped"

    Oxiland provides Redland-shaped workflows. Tip **0.12** retains the frozen
    0.11 source/binary compatibility evidence and adds a strict host-scoped
    performance win; it is not an rdflib adapter. Start with the
    [positioning guide](evaluators/positioning.md) and verify each claim in the
    [parity ledger](parity.md).

<div class="home-footer-cta" markdown>

### Build your first dataset

Choose the language you already use. Both tracks cover installation, models,
RDF I/O, SPARQL, persistence, failure handling, and production operations.

[Python quick start](users/python.md){ .md-button .md-button--primary }
[Rust quick start](users/rust.md){ .md-button }

</div>
