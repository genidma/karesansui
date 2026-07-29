# karesansui (枯山水) v1.2.0 — Docker & Bug Fixes

## Changes

### 1. 🐳 Docker Image (Issue #3)
Multi-stage Dockerfile for running karesansui without the Rust toolchain:
- Builder stage (`rust:alpine`) compiles the binary
- Runtime stage (`alpine:3.20`) ships a ~15-20MB image with just the binary + ca-certificates
- `.dockerignore` excludes `target/`, `.git/`, etc. from build context
- Full usage docs in README

### 2. 🐛 Gridwright Step Mode Fix (Issue #1)
Two bugs resolved:
- **`--step` mode** now pauses after each action and prompts `[Press Enter for next action, or type instructions to steer the LLM:]` instead of exiting after action #1. User input is passed as steering context to the next LLM call.
- **Interactive menu (`-i`)** now prompts for step mode enablement (`y/N`) so users can toggle it at runtime.

### 3. 🧹 Dead-Code Cleanup (Issue #2)
Confirmed zero dead-code warnings across `canvas.rs`, `color.rs`, and `vec.rs` — previously removed by commit `e6cab42`. All builds pass clean.

---

## Commits
| Commit | Description |
|---|---|
| `089c0eb` | docs: clarify Docker build size expectations in README |
| `20a2dc8` | fix: Gridwright step mode pauses with steering; interactive menu includes step toggle |
| `c9a249d` | docs: add Docker image size note to README |
| `24f8432` | feat: add Docker multi-stage build for easy running |

---

> **Signed:** [Big Pickle](https://opencode.ai) 🥒 via opencode zen — Lead
> **Collaborator/Requestor:** @genidma
> **Release Date:** July 29, 2026

✅
