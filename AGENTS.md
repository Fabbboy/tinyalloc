# Repository Guidelines

## Project Structure & Module Organization
The root library (`src/lib.rs`) coordinates initialization helpers in `src/init.rs` and FFI exports in `src/ffi.rs`. Core allocator variants live in workspace crates under `crates/`, matching the architecture summary in `CLAUDE.md` (bitmap, list, array, sys, alloc). C-facing glue is defined in `init.c` plus `tinyalloc.h`; `build.rs` regenerates headers via `cbindgen`. Benchmarks reside in `benches/` with Criterion harnesses, examples in `examples/`, and shared config at the workspace root (`rustfmt.toml`, `rust-toolchain.toml`).

## Build, Test, and Development Commands
- `cargo build` / `cargo build --release` – compile the workspace; release ensures header regeneration.
- `cargo test` – run unit + doc tests for every crate.
- `cargo test -p tinyalloc-alloc` – focus on a specific allocator while iterating.
- `cargo bench --bench allocator_bench` – execute Criterion perf baselines after allocator changes.
- `cargo fmt` and `cargo clippy --all-targets --all-features` – enforce style and lint gates before PRs.

## Coding Style & Naming Conventions
Follow the pinned nightly toolchain and the repository `rustfmt.toml` (four-space indentation). Prefer `CamelCase` for types, `snake_case` for items, and `SCREAMING_SNAKE_CASE` for constants. Import symbols instead of using fully qualified paths, keep functions small, and limit comments to explaining intent. Expose items with the narrowest visibility (`pub(crate)`, `pub(super)`), rely on the `getset` crate for accessor patterns, and scope `unsafe` blocks tightly with rationale. Avoid “magic numbers”; convert them into named `const` values derived from allocator parameters.

## Testing Guidelines
Place unit tests next to the modules under test and add doc tests for public APIs. Cover both successful allocations and exhaustion or failure paths, mirroring scenarios described in `CLAUDE.md`. Integration and FFI behavior should live in `examples/` or dedicated tests. Re-run `cargo test` across the workspace before submitting; re-run targeted crates and Criterion benches when making performance-sensitive changes.

## Commit & Pull Request Guidelines
Historical commits are terse; adopt imperative subjects ≤60 characters and include optional body context plus benchmark notes. Reference issues with `#id`, and mention regenerated artifacts (`tinyalloc.h`) or alignment tools. Pull requests must summarize intent, list validation commands (`cargo fmt`, `cargo test`, key benches), call out ABI or allocator changes, and request review from maintainers of affected crates.

## Maintainer Rules & Safety Expectations
Use `NonNull<T>` for non-null pointers and `Option<NonNull<T>>` when null is valid. Prefer `Result<T, E>` for fallible APIs, and use `NonZero*` types for counts and sizes. Be explicit with lifetimes and avoid manual dependency edits—use `cargo add` or workspace tables. Ensure Rust and C APIs stay in sync; after public changes run `cargo build --release` and confirm `tinyalloc.h` plus `init.c` reflect the updated ABI.

## AI Agent Workflow
- Treat each engagement as an incremental change request: inspect the repo state, restate assumptions, then execute in small, reviewable steps.
- Default to `cargo fmt`, targeted `cargo test`, and relevant benches before handing results back; call out skipped checks and why.
- Prefer reading existing modules and CLAUDE.md before designing new abstractions so additions align with the documented architecture.
- Use repository tooling (e.g., `cbindgen` via `build.rs`) rather than ad-hoc scripts when generating code or headers.

## Task Handoff & Communication
- Surface design uncertainties early, especially around allocator invariants, cross-platform mapper behavior, or FFI ABI changes.
- Summarize file-level edits and rationale in the final message; mention follow-up work or validation the maintainer should run.
- When a task requires user decisions (e.g., trade-offs between eager commit vs. lazy map), propose concise options instead of guessing.
- Respect existing TODOs and open issues; if new gaps are found, note them in the response rather than silently leaving placeholders.

## Safety & Validation for Agents
- Treat unsafe blocks as hot spots: explain why they remain sound and reference invariants upheld elsewhere in the code.
- Avoid invoking forbidden symbols (e.g., `__libc_malloc`) and keep allocator independence intact when wiring fallback behaviors.
- Document any mapper recursion or bootstrap heuristics you introduce, including guard rails to prevent re-entrancy loops.
- If sandbox or permission constraints block execution, state the limitation and provide manual steps for the maintainer to reproduce.
