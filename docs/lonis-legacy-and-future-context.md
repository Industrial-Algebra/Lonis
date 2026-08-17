# Lonis: Legacy Prototype and Future Direction

## Current Reality

This repository currently contains the original Python implementation of **Lonis**: a bitmap-to-structured-design-data analyzer.

Its purpose is to measure image properties directly and emit structured JSON, rather than relying on an AI vision model to guess at colors, layout, gradients, edges, or textures.

Core pipeline:

```text
image -> color -> spatial -> edge -> gradient -> texture -> JSON
```

This implementation remains valuable and should be preserved in a functioning state for:

- demos
- historical reference
- comparison against later systems
- validation of the original concept

## Important Reframing

The current Python Lonis is **not** the long-term architectural destination.

It is the validated prototype that established:

- which low-level measurements matter
- where flat JSON summaries are useful
- where flat summaries stop being sufficient
- why a richer perception model is needed

## Perceptron

**Perceptron** is the next-stage perception model.

Its goal is not merely to summarize bitmap measurements, but to produce structured observations rich enough for an LLM or other AI system to reason from.

Perceptron does **not** perform the final inference itself. Instead, it provides sensory descriptions that support inference.

The intended output model is based on **P-expressions**: EDN / s-expression-like structures that record observations and the path of attention used to deepen them.

This enables iterative examination rather than one-shot summary.

In short:

- Lonis Python measures pixels well
- Perceptron structures perception more deeply

## The New Lonis Direction

The name **Lonis** is also being retained, but with a different role.

Rather than remaining only a Python image-analysis CLI, Lonis will become a **pluggable, AI-oriented command-line tool harness** organized as a **Rust monorepo**.

This new Lonis will provide access to tools and mechanics that can produce outputs for LLM consumption, including Perceptron descriptions.

### Why this approach

This harness model has proven preferable to MCP in many cases because it offers:

- well-defined tool surfaces
- local explicit invocation
- less context pollution
- less annoying configuration burden
- cleaner boundaries between tools and consuming agents

The point is not to expose everything through a persistent protocol server, but to provide sharply bounded AI-facing commands that are easy to compose and reason about.

## How the Pieces Relate

### Legacy Python Lonis

Role:
- archived prototype
- measurement-oriented bitmap analyzer
- JSON output
- preserved for demos and reference

### Perceptron

Role:
- structured sensory input engine
- perceptual description model
- EDN / s-expression / P-expression output
- designed for LLM interpretation and iterative attention

### New Lonis Harness

Role:
- Rust monorepo
- primary command-line harness crate
- additional crates for supporting tools
- delivery surface for Perceptron and other AI-oriented capabilities

## Intended Transition

The project should be understood as moving through these stages:

1. **Lonis (Python prototype)**
   - proved the value of direct measurement
   - demonstrated the limits of flat JSON summaries

2. **Perceptron (new perception model)**
   - introduces richer, structured, attention-aware descriptions

3. **Lonis (Rust harness/monorepo)**
   - becomes the platform through which Perceptron and related tools are exposed

## Guidance for Readers

If you are looking at the current Python codebase:

- treat it as a preserved legacy implementation
- expect it to remain runnable and understandable
- do not treat it as the primary destination for new architecture work

If you are thinking about the future of the project:

- Perceptron is the key perception-model pivot
- the new Lonis is the key tooling/platform pivot

## Summary

The project is no longer a single thing.

- **Old Lonis** is the preserved Python prototype.
- **Perceptron** is the structured perception engine.
- **New Lonis** is the Rust command-line harness platform that will expose Perceptron and other AI-facing tools.

This split is intentional and should be reflected in future planning and documentation.
