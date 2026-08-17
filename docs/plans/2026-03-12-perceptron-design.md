# Perceptron: An AI Sensory Input Engine

## Origin

Lonis began as a bitmap-to-design-data analyzer — 5 analyzers (color, spatial, edge, gradient, texture) that measure pixel-level properties and emit flat JSON. Testing against a 16-faceted glass prism with retroreflection and caustics exposed fundamental limitations: the system summarizes pixels rather than capturing the structured observations an AI needs to reason about an image.

The name reclaims Rosenblatt's 1958 coinage — "instrument of perception" — for what it was always meant to be.

## Core Insight

**The system should output structured observations rich enough to support inference, without performing the inference itself.**

Not "caustic detected" (too interpreted). Not "pixels are white" (too raw). But "bright linear features in shadow region, aligned with edges of adjacent transparent object, tapering from contact point" — observations that *allow* an AI to reason toward caustics, or toward an alternative explanation if context suggests otherwise.

The key phrase: **"inferring the physical properties that produced the image, rather than summarizing the image's pixels."**

But critically — Perceptron doesn't infer those properties. It provides the structured sensory data from which an AI *can* infer them. Measurement, not interpretation. Painters reproduced caustics for centuries before the physics were modeled. They observed light patterns and reproduced them. Perceptron captures what a careful observer would notice.

## The Contemplation Problem

Human visual understanding is iterative:

1. **Glance** — dark scene, grid of objects, teal thing catches the eye
2. **Attend** — the teal thing has structure, repeating features around a circle
3. **Examine** — bright lines below it, in the shadow — that's odd
4. **Ponder** — the lines follow the object's edges, taper with distance
5. **Realize** — light is moving *through* it

Each pass is guided by the previous. Current AI vision does pass 1. Lonis does a different pass 1 with measurements. Nobody does passes 2-5 — the iterative deepening where each observation guides the next look.

A World Model (continuous latent scene representation) would solve this natively but is beyond current publicly accessible AI. Perceptron approximates the iterative process through progressively structured views accumulated in an AI's context window.

## P-Expressions

The notation is a pseudo-LISP inspired by Clojure's approach to data (homoiconic, composable, lazy). Each s-expression is a branch to examine the media in another dimension. The tree structure models the contemplation process itself.

```clojure
(perceive image/gem-prism
  (shape circular :r 103
    (attend → (periodic :fold 16 :element half-cylinder)))
  (material teal-transparent
    (attend → (transmissive :ior-range [1.4 1.6]
      (attend → (retroreflective :along element-edges)))))
  (shadow
    (contains bright-linear
      (attend → (aligned-with (edges parent))
        (attend → (attenuation (decay-from contact-point)
          (suggests caustic-transport)))))))
```

Key properties:

- **Each `attend` is a deeper pass** — the outer expression is what you notice first, deeper levels require directed attention
- **The tree records the path of attention**, not just the final answer
- **The analysis format and query format are the same language** — an AI reads the first pass, then writes a request for deeper examination in the same notation
- **`suggests` not `detects`** — inference emerges from depth of examination, not classification
- **Modality-agnostic** — the same structure works for sound, or any signal:

```clojure
(perceive audio/sample
  (pitch
    (fundamental A4 :confidence 0.7)
    (attend → (A4+50cents :confidence 0.9)
      (attend → (subharmonic A3 :amplitude -18dB
        (attend → (beating :rate 2Hz :with fundamental)))))))
```

## Architecture: The Retinal Stack

The architecture mirrors biological vision. The retina isn't a camera — it has ~130M photoreceptors feeding through five types of interneurons into ~1.2M ganglion cells, all *before* signals reach the brain. The optic nerve transmits a preprocessed, multi-channel feature stream, not an image.

| Biology | Perceptron | Function |
|---------|-----------|----------|
| Photoreceptors | CV analyzers (Lonis legacy) | Pixel-level measurement: color, edges, luminance, texture |
| Retinal interneurons | Traditional CV + learned representations | Symmetry detection, material classification, spatial relationships, frequency analysis, latent feature encoding |
| Ganglion cells | Composition model | Formats observations into p-expressions — structures perception, doesn't reason about it |
| Optic nerve | MCP interface | Delivers structured perception to the "cortex" (Claude or other AI) |

The consuming AI orchestrates iterative deepening: reads first-pass p-expressions, identifies where to `attend`, sends queries back through MCP, triggering targeted re-analysis that produces deeper p-expressions.

## Language: Rust

Python Lonis is archived as a validated prototype. Perceptron is a Rust project for:

- **Amari integration** — geometric algebra operations on perception data without serialization boundaries
- **Performance** — each `attend` pass must be fast enough to feel interactive
- **WASM compilation** — same code runs native, in browser, or as MCP server
- **Ecosystem consistency** — fits into the existing Rust/Amari toolchain

## Relationship to Existing Work

- **Lonis (Python)** — archived prototype. Its measurements validated what matters; its limitations defined Perceptron's requirements
- **Amari** (geometric algebra library) — mathematical substrate for spatial relationships, symmetries, and transforms
- **Amari-MCP** — existing MCP infrastructure to build on
- **The MCP server** — the iterative dialogue interface and `attend` query mechanism

## Build Roadmap

### Phase 1: CPU-only (current hardware)
- Rust port of CV layer — edge detection, color analysis, contour extraction, symmetry detection (`opencv` crate or pure-Rust `imageproc`)
- P-expression grammar, parser, and emitter
- MCP server skeleton with `attend` protocol
- Amari integration for geometric representations of shape, symmetry, spatial relationships
- Archive Python Lonis

### Phase 2: GPU-enabled (RTX laptop or 1080Ti)
- CLIP encoder via ONNX runtime (`ort` crate) for latent representations
- Stable Diffusion VAE encoder for learned perception features
- Local model for p-expression composition (or route to Claude via MCP)
- Recursive re-encoding: encode whole image → identify regions of interest from latent space → re-encode at higher resolution → repeat (computational analog of saccadic eye movement)

### Phase 3: Full pipeline
- Larger models, full iterative deepening at interactive speed
- Multi-modal extension (audio, other sensory data)
- Integration with world model capabilities as they become publicly accessible

## Test Case

The reference object for validating the design: a circular glass prism with 16 regular rounded half-cylinders around the perimeter, teal glass with retroreflection and radiosity, casting caustics in its shadow where light is transported down the edges, with a flat transparent face reflecting the environment along a gradient.

If Perceptron can give an AI enough structured data to reason about *this* object, it can handle anything.

## Status

This document captures the design conversation of 2026-03-12. The notation, architecture, roadmap, and Rust migration decision are established. P-expression grammar and Phase 1 project structure are next.
