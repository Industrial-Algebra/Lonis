# Lonis Legacy Archive Plan

## Purpose

Preserve the existing Python implementation of Lonis in a stable, functioning state for:

- legacy reference
- demos
- comparison against Perceptron
- validation of the original bitmap-analysis concept

This archive is intentional. The Python Lonis prototype is not a failed branch to be discarded; it is a completed proof-of-concept whose strengths and limitations informed the next stage of the project.

## Scope of the Legacy Archive

The archived Lonis should remain usable as a standalone tool that:

- installs cleanly
- runs from the CLI
- analyzes images end-to-end
- emits structured JSON
- retains its tests and fixtures
- remains understandable to future readers

The archive target is the current Phase 1 system:

```text
image -> color -> spatial -> edge -> gradient -> texture -> JSON
```

## What Must Be Preserved

### Functional surface

- `lonis analyze <image>` CLI
- `--only`, `-o`, and `-v` flags
- analyzer pipeline orchestration
- JSON output schema
- sample fixtures and integration test structure

### Code structure

- `analyzer/cli.py`
- `analyzer/pipeline.py`
- `analyzer/analyzers/*`
- `analyzer/utils/image.py`
- `tests/*`
- `README.md`
- `pyproject.toml`
- `requirements.txt`

### Documentation value

The archive should clearly communicate:

- what Lonis does
- why it mattered
- why it is being archived rather than extended
- how it relates to Perceptron and the new Lonis harness

## Archive Principles

1. **Keep it working**
   - Do not archive in a half-broken or dependency-drifting state.

2. **Do not continue feature expansion**
   - Bugfixes for archival stability are acceptable.
   - New conceptual work belongs in Perceptron or the new Lonis harness.

3. **Preserve historical clarity**
   - Readers should understand that this was the original bitmap measurement prototype.

4. **Minimize churn after archival**
   - The more this code changes after being archived, the less useful it becomes as a historical and comparative artifact.

## Definition of "Archived in a Functioning State"

The Python Lonis archive is complete when all of the following are true:

### 1. Environment is reproducible

Document a known-good setup path, including:

- supported Python version
- dependency installation
- fixture generation expectations, if needed
- test invocation

### 2. CLI works

These should succeed in a clean environment:

```bash
pip install -e .
lonis analyze tests/fixtures/grid_layout.png
lonis analyze tests/fixtures/grid_layout.png --only color,edge
lonis analyze tests/fixtures/grid_layout.png -o out.json
```

### 3. Tests pass

At minimum:

```bash
python3 -m pytest -v
```

If the external integration image is unavailable, the suite should still remain useful via skip behavior.

### 4. Docs are explicit

The repo should say clearly that:

- this Python implementation is the legacy prototype
- it is preserved intentionally
- active development has shifted elsewhere

### 5. A final archival marker exists

Create a durable archival reference such as:

- a Git tag
- a release
- or a clearly named branch boundary

Suggested names:

- `lonis-python-legacy-v0.1.0`
- `legacy-lonis-final`
- `python-prototype-archive`

## Recommended Archive Tasks

### Task 1: Stabilize dependencies

- ensure `pyproject.toml` and `requirements.txt` are aligned
- verify installation in a clean environment
- consider pinning or constraining versions if breakage risk is high

### Task 2: Verify end-to-end behavior

- run the CLI on fixture images
- confirm JSON shape matches docs
- ensure file output works
- ensure invalid analyzer handling is correct

### Task 3: Run and clean test suite

- run full tests
- fix any environment-sensitive failures
- confirm integration test skip behavior is intentional and documented

### Task 4: Add archive messaging

Update top-level documentation to state:

- Python Lonis is archived as a legacy/demo prototype
- Perceptron is the successor perception model
- the new Lonis direction is an AI-oriented tool harness monorepo

### Task 5: Tag archival state

After validation:

```bash
git tag lonis-python-legacy-v0.1.0
```

or equivalent.

## Non-Goals for the Legacy Archive

Do not turn the archived Python Lonis into:

- the future Perceptron implementation
- the new Rust harness
- a partial migration target
- a constantly evolving compatibility layer

That would blur the line between prototype and successor.

## Relationship to Future Work

### Legacy Python Lonis

- flat JSON bitmap-analysis pipeline
- deterministic measurement-oriented prototype
- retained for demos, reference, and comparison

### Perceptron

- structured sensory input engine
- emits P-expressions / EDN-like perceptual descriptions
- designed for LLM consumption and iterative attention

### New Lonis

- Rust monorepo
- pluggable AI-oriented command-line harness
- exposes Perceptron and other tool surfaces without MCP-style context/config overhead

## Archive Outcome

When this plan is complete, the project should have:

- a stable preserved Python Lonis
- a clear conceptual handoff to Perceptron
- a clear platform handoff to the new Lonis harness
- a clean historical record of the transition
