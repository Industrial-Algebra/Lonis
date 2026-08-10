# Planning Documents Index

This directory contains both the original Lonis planning documents and the newer documents describing the Perceptron and Lonis-harness pivot.

## Reading Guide

### If you want the current strategic picture
Start here:

1. `2026-03-28-figma-first-slice-contract-pack.md`
   - concrete first-slice contract target for the Rust Figma provider rewrite
   - defines representative tool contracts, diagnostics schemas, and safety expectations

2. `2026-03-28-figma-rust-provider-rewrite-strategy.md`
   - decision record and kickoff strategy for rewriting the Figma surface as a Rust provider
   - includes a ready-to-use prompt for another agent to begin the rewrite analysis

3. `2026-03-28-lonis-external-provider-protocol-spec-v0.md`
   - executable-provider protocol draft for external tool surfaces
   - defines required provider subcommands, JSON wire format, stdout/stderr discipline, and exit semantics

3. `2026-03-28-lonis-provider-interface-spec-v0.md`
   - first-pass provider/interface spec for how Lonis discovers, describes, and invokes tool surfaces
   - defines the canonical `lonis call <tool>` model and normalized envelopes
   - uses Perceptron and Figma as the key reference provider classes

3. `2026-03-28-lonis-figma-provider-mapping-draft.md`
   - maps Kai Jyozu concepts and operational lessons into the Lonis provider model
   - identifies Figma as an early proving ground for verification, safety, and diagnostics design

4. `2026-03-28-lonis-harness-design-draft.md`
   - first-pass design draft for the new **Lonis** as an AI-native tool harness
   - reflects the updated decision that Perceptron is a separate project consumed by Lonis
   - frames Figma integration as another external tool surface/provider

3. `2026-03-12-perceptron-design.md`
   - defines **Perceptron** as the new structured perception model
   - introduces P-expressions / EDN-like perceptual descriptions
   - captures the perception-side pivot that motivated the broader redesign

3. `2026-03-28-lonis-legacy-archive-plan.md`
   - explains how the existing Python Lonis should be preserved as a working legacy/demo prototype

4. `../lonis-legacy-and-future-context.md`
   - explains the split between:
     - legacy Python Lonis
     - Perceptron
     - future Lonis as an AI-oriented command-line harness monorepo

## Historical Documents

### `2026-03-06-lonis-design.md`
The original design for Lonis as a bitmap-to-structured-design-data analyzer.

Use this to understand:
- the original problem statement
- the five analyzer pipeline
- the JSON-oriented output model
- the initial semantic/provider roadmap

### `2026-03-06-lonis-implementation.md`
The original implementation plan for the Python prototype.

Use this to understand:
- how the existing codebase was intended to be built
- test and fixture structure
- the Phase 1 pipeline implementation details

## How to interpret these documents together

The documents in this folder represent an evolution, not a single stable plan.

### Earlier phase
Lonis was conceived as:
- a Python bitmap-analysis CLI
- a measurement-oriented pipeline
- a producer of structured JSON

### Current pivot
The project direction now separates into:

- **Legacy Python Lonis**
  - preserved prototype
  - demo/reference implementation

- **Perceptron**
  - separate project/repo
  - structured sensory input engine
  - P-expression / EDN-like output

- **Future Lonis**
  - Rust monorepo
  - strict AI-oriented command-line tool harness
  - delivery surface for Perceptron, Figma integration, and other tool providers

## Recommended interpretation order

If you are new to the repo, read in this order:

1. `../lonis-legacy-and-future-context.md`
2. `2026-03-28-figma-first-slice-contract-pack.md`
3. `2026-03-28-figma-rust-provider-rewrite-strategy.md`
4. `2026-03-28-lonis-external-provider-protocol-spec-v0.md`
5. `2026-03-28-lonis-provider-interface-spec-v0.md`
6. `2026-03-28-lonis-figma-provider-mapping-draft.md`
7. `2026-03-28-lonis-harness-design-draft.md`
8. `2026-03-12-perceptron-design.md`
9. `2026-03-28-lonis-legacy-archive-plan.md`
10. `2026-03-06-lonis-design.md`
11. `2026-03-06-lonis-implementation.md`
