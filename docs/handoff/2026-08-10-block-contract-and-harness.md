# Lonis Handoff — Block Contract + Harness Runtime

Date: 2026-08-10
From: Dominic session (doctrine §8.A resolution)
Branch to start on: `develop` (HEAD `5004d1c`, clean)

## Why this handoff exists

The Anima Ecosystem Doctrine resolved the **block taxonomy** (§8.A → §2.7) on
2026-08-10. That resolution is now the authoritative spec for what a structured
domain object *is*, and it is grounded in `amari-discovery` as the reference
implementation. This handoff turns that spec into Lonis implementation work.

**Start here, not by re-deriving the design.** Read the doctrine §2.7 first.

## Where Lonis stands now

- `develop` @ `5004d1c`. 36 workspace tests, clippy + `cargo doc` clean,
  dual-push (GitHub `origin` + Forgejo `king-ghidorah`).
- Workspace crates (Apache-2.0, edition 2021, MSRV floor 1.75):
  - **lonis-schema** — `Capabilities` trait (5 methods: `id`/`name`/
    `description`/`input_schema`/`output_schema`), JSON-typed I/O contract.
  - **lonis-derive** — `#[derive(LonisCapabilities)]` from
    `#[lonis(tool_id = "...")]`; re-exported from lonis-schema behind `derive`
    (serde-style).
  - **lonis-core** — `Tool` trait (`Capabilities` + JSON-typed `invoke`),
    `ToolRegistry`, `render`/`render_error` per `OutputMode`, `run_tool`
    (success→stdout, error→stderr + nonzero exit).
  - **lonis-cli** — `lonis` binary: `tools list`/`describe`, `call <id> [input]`,
    `--mode`; builtins `echo` + `version`.
- `main` @ `348c555` = the old Python vision code, tagged
  `lonis-python-legacy-v0.1.0` (vision was spun out to Perceptron).

## The doctrine grounding (authoritative)

`IA-documents/ANIMA_ECOSYSTEM_DOCTRINE.md` §2.7 (`@ 9f6b890`, in the IA-documents
repo) defines:

- **A block** (structured domain object) carries six cross-cutting properties:
  1. **Envelope** — `{ schema_version, provenance, warnings, data }`.
  2. **Attribution** — `{ identity, viewpoint, provenance { when, where, producer } }`.
     *Richer* than `amari-discovery`'s `capability_id` (which collapses it because
     amari-discovery is single-agent/capability-scoped). Lonis keeps the full slot
     (multi-participant, stream-scoped).
  3. **Bounded** — resource limits / `maxItems` / max-length are first-class.
  4. **Versioned** — `schema_version` + protocol marker + a stable `$id` per kind.
  5. **Replayable** — content hashes (`project_hash`/`input_hash`/`plan_hash`/
     `result_hash`) + seed → deterministic replay.
  6. **Render-parity** — human + machine render from the *same* typed object.
- **14-kind seed corpus**, three categories (evolvable — add/prune as Lonis is
  implemented):
  - *Participant-stream primitives* (7): `message`, `question`, `answer`,
    `decision`, `action`, `assumption`, `summary`.
  - *Knowledge/definition* (4): `evidence`, `definition`, `capability`, `intent`.
  - *Process* (3): `plan`, `result`, `outcome`/`error`.
- **Dogfooding lineage**: `amari-discovery` = one *vertical* instance (dogfoods
  `amari-holographic`; implements the knowledge + process subset). Future
  `karpal-discovery` = second vertical (dogfoods `karpal`). **Lonis is the
  *horizontal* generalization** — the harness that hosts tools and carries the
  general block contract any tool emits through. Lonis is **not** a discovery
  tool; it is intended to be *more* capable than `amari-discovery`.

## Reference implementation — examine before coding

`~/working/industrial-algebra/amari/amari-discovery` is the reference. Look at:

- `schemas/{goal,plan,probe,response,request}-v1.json` — the canonical wire shapes.
  (`response-v1.json` is the envelope; `plan-v1.json` shows a tagged-union
  `PlanStep` with 6 variants.)
- `src/protocol.rs` — `Envelope`, `Provenance`, `CandidatePlan`, `PlanStep`,
  `Recommendation`, `ProbeResult`, `CapabilityId`, `DiscoveryError`.
- `src/capabilities.rs` — `Capabilities` (self-describe) + `ResourceLimits`.
- `src/schema.rs` — the `SchemaKind` / `ProtocolSchema` registry pattern.

Run it to see live structured output: `amari capabilities --json`,
`amari schema response`, `amari inspect <rust-pkg> --json` (rich `ProjectSnapshot`),
`amari discover search holographic --json`, and a JSON-mode error (`amari recommend
/bad --goal x --json 2>&1` → `{kind, message, details}` + stable exit code).

amari-discovery implements the **knowledge/definition + process** kinds; Lonis
generalizes to **all 14** plus the richer attribution.

## The next work (priority order)

### 1. Implement the `Block` contract in `lonis-schema` (the §2.7 payoff)
The concrete types, the 14-kind enum behind `BlockKind`:

- `Block { envelope, attribution, bounds, payload: BlockKind }`
- `Envelope { schema_version, provenance, warnings, data }`
- `Attribution { identity, viewpoint, provenance: BlockProvenance { when, where, producer } }`
- `BlockKind` — an **extensible** enum; seed = the 14 (three categories).
- Every `Block` uniformly versioned / replayable (content hash + seed) /
  render-parity (human + JSON from one typed value). Mirror amari-discovery's
  `$id`-per-kind + `schema_version` discipline; emit stable JSON Schemas like
  amari-discovery's `schemas/` if useful.
- TDD (IA standards): serde round-trips, render-parity property tests.

### 2. Reconcile `Block` with the existing `Tool` / `Capabilities` contract
**Open design question for this session to settle** (do not assume): does a tool's
`invoke` return `Block`s? Likely yes — the `Block` becomes the canonical tool
*output*; `Capabilities` / `Tool` remain the tool-*registration* contract. Decide
whether `invoke` returns one `Block`, a `Vec<Block>`, or a stream (for ndjson
parity with amari-discovery). Document the decision in an ADR under `docs/adr/`.

### 3. External subprocess tools (the real composable model)
`SubprocessTool` implementing `Tool` by spawning composable external CLIs. This is
the Lonis thesis in action — *"MCP exposes servers to models; Lonis exposes tools
to agents."* A subprocess adapter is how Lonis hosts arbitrary tools. Must be
bounded (timeouts, output limits) per the `Block` `bounds`.

### 4. `lonis` umbrella facade crate
Re-export schema + derive + core (serde-style): `lonis::Block`,
`lonis::Capabilities`, `lonis::LonisCapabilities`. One crate for consumers.

## Constraints (IA standards)

Apache-2.0 with license headers. TDD. Zero clippy warnings
(`cargo clippy --all-features --all-targets -- -D warnings`). `cargo doc` clean.
`channel = "stable"` (`rust-toolchain.toml`). Gitflow: feature branches → PR →
`develop`. Dual-push GitHub + Forgejo (`king-ghidorah`).

## Pointers

- **Doctrine** (IA-documents repo): §2.7 block taxonomy; §2.6 identity/skill/
  presence; §3 Lonis = Tools plane.
- **Reference impl**: `amari/amari-discovery` (`schemas/`, `src/protocol.rs`,
  `src/capabilities.rs`, `src/schema.rs`).
- **Existing Lonis plan**: `docs/plans/lonis-schema-design.md`.
- **Memory**: `mem_3e10fa11` (Lonis harness runtime), `mem_71d8a04d`
  (block taxonomy / §8.A resolution).
