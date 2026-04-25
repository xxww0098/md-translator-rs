# AGENTS.md

This file is the operating contract for coding agents in this repo.

## Project
- Name: md-translator-rs
- Type: Rust workspace with a root crate
- Edition: 2024
- Surfaces: CLI (`mdxlate`) + library (`MdTranslator`)

## Layout
- `Cargo.toml` / `Cargo.lock`: crate metadata and lockfile
- `LICENSE`: MIT license text
- `providers.yaml.example`: example YAML config for providers (`deeplx`, `ai`, `deepl`, `performance`, `cache`, `markdown`), including adaptive concurrency runtime knobs
- `run_test.sh`: root-level launcher for the `live-provider-smoke` workspace member
- `examples/live-provider-smoke/`: standalone Rust caller project for real provider smoke runs
  - `Cargo.toml`: consumer crate manifest using the root crate via `path`
  - `src/main.rs`: real-provider smoke runner built on the library API
- `test_md/`: source markdown/skill fixtures used to prepare translation inputs
- `test_input/`: root-level input directory scanned by `live-provider-smoke` for `.md` files
- `test_output/`: root-level output directory receiving translated copies from `test_input/`
- `src/main.rs`: CLI entrypoint
- `src/lib.rs`: library entrypoint (re-exports all public modules)
- `src/bin/deeplx_export/`: auxiliary binary (accepts `<INPUT> <OUTPUT>` positional args)
- `src/ast/`: comrak-backed Markdown AST pipeline
  - `parser.rs`: MarkdownAst wrapper, parse_markdown helpers
  - `extractor.rs`: TranslatableNode extraction from AST
  - `batch.rs`: XML segment packing for AI provider batching
  - `parse_response.rs`: XML response parsing from AI providers
  - `render.rs`: AST node replacement and markdown rendering
  - `pipeline.rs`: parse → extract → translate → replace → render orchestrator
- `src/cache/`: two-tier cache implementation
  - `mod.rs`: Cache trait, deprecated FileCache
  - `memory.rs`: moka-based in-memory cache with TTL
  - `disk.rs`: sled-based persistent cache
  - `two_tier.rs`: memory → disk → miss lookup chain
- `src/provider/`: TranslationBackend implementations
  - `mod.rs`: provider exports and `TranslationBackend` trait
  - `concurrent.rs`: shared provider-side scheduling contract for provider-specific work units
  - `gtx.rs`: Google Translate web backend
  - `openai_compat.rs`: OpenAI-compatible XML batch backend (auto-appends `/v1/chat/completions` to base URL)
  - `deepl.rs`: DeepL native batching backend
  - `deeplx.rs`: DeepLX per-item request backend
  - `xml.rs`: provider-local XML batch packing/parsing helpers
- `src/config.rs`: YAML config parsing (`deeplx`, `ai`, `deepl`, performance/cache/markdown, adaptive concurrency defaults, resolved provider settings)
- `src/cli.rs`: CLI argument parsing
- `src/client.rs`: reqwest Client builder with tuned connection pool
- `src/dedup.rs`: request deduplication layer using broadcast channel
- `src/fallback.rs`: MultiProviderBackend with per-document fallback
- `src/reporting.rs`: TranslationReporter and ReporterState for progress tracking
- `src/engine.rs`: MdTranslator core pipeline
- `src/markdown.rs`: legacy placeholder-based Markdown protection
- `src/types.rs`: TranslateOptions, BatchItem, TranslateRequest/Response types and adaptive runtime options
- `src/error.rs`: MdTranslatorError, including structured provider HTTP failures with retry hints
- `tests/`: integration and mock tests
  - `provider_smoke.rs`: end-to-end smoke tests for all 4 translation backends (GTX, DeepLX, OpenAI-Compat, DeepL)
- `docs/Error.md`: hard-bug log

## Stack
- Rust + Cargo + `httpdate` for parsing `Retry-After` headers
- Verify with: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-targets --all-features`, `cargo build --workspace`, `cargo doc --workspace`

## Hard Rules
- If project rules, structure, workflow, or stack change: update this file FIRST.
- If repo structure changes: update this file FIRST.
- Do NOT change code first and docs later.
- Do NOT refactor unrelated code during a fix.
- Do NOT claim success without running verification.
- Make the smallest correct change.

## Required Order
1. inspect repo context
2. update `AGENTS.md`
3. update code
4. run verification
5. update docs if needed

## Docs Sync
- Update `README.md` for user-facing changes.
- Update `docs/Error.md` for hard bugs.

## Hard Bug Rule
If a bug is non-obvious, expensive to diagnose, or likely to recur, it MUST be recorded in `docs/Error.md` before the task is closed.

## `docs/Error.md` Entry Format
- Date:
- Area:
- Symptom:
- Root cause:
- Resolution:
- Prevention:
- Related files:
