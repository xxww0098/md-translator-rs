# Error Log

Use this file for hard bugs, non-obvious failures, and recurring debugging knowledge.

Record issues here when the root cause was not obvious, the debugging cost was meaningful, or the same class of bug is likely to happen again.

Do not use this file for trivial typos, obvious compile errors, or one-off mistakes with no reuse value.

## When to add an entry

Add an entry when any of these are true:

- the root cause was non-obvious
- multiple failed debugging paths were tried
- parser, async, batching, cache, or provider boundaries were involved
- environment or tooling behavior was misleading
- a future agent would likely waste time rediscovering the same fix

## Entry format

```md
## [Short bug title]

- Date:
- Area:
- Symptom:
- Root cause:
- Resolution:
- Prevention:
- Related files:
```

Optional when useful:

```md
- Reproduction:
- False leads:
- Notes:
```

---

## Example

## Placeholder restoration broke across translation batches

- Date: 2026-04-22
- Area: markdown / engine
- Symptom: fenced code blocks and inline code were partially translated even though they should have been preserved
- Root cause: placeholder restoration order was inconsistent after text was split into multiple translation batches
- Resolution: normalize placeholder bookkeeping before batching and restore placeholders in original document order
- Prevention: add regression tests covering mixed markdown, inline code, and fenced code across multi-batch input
- Related files: `src/markdown.rs`, `src/engine.rs`, `tests/`
- Reproduction: translate a markdown file with multiple fenced blocks and enough content to trigger batching
- False leads: initially suspected provider output drift, but the bug was caused by local placeholder handling

## Runtime markdown path bypassed AST pipeline

- Date: 2026-04-22
- Area: engine / cli / cache
- Symptom: reviewers correctly found that shipped markdown translation still used the legacy placeholder+line path and CLI still defaulted to deprecated `FileCache`
- Root cause: AST modules and `TwoTierCache` were implemented, but the production `MdTranslator::translate_markdown_with_report` and CLI wiring were never switched over from the earlier compatibility path
- Resolution: routed markdown runtime translation through `AstTranslationPipeline`, translated extracted AST nodes directly, and changed shipped binaries to instantiate `TwoTierCache`; also removed hardcoded OpenAI/DeepL endpoint fallbacks so runtime config must come from CLI/env/YAML
- Prevention: add runtime regression coverage that inspects actual markdown request batches instead of only testing AST helpers in isolation, and keep CLI wiring tests aligned with reviewer acceptance criteria
- Related files: `src/engine.rs`, `src/cli.rs`, `src/bin/deeplx_export/main.rs`, `tests/backward_compat.rs`, `README.md`

## YAML provider settings were parsed but ignored by shipped backend wiring

- Date: 2026-04-22
- Area: cli / config / provider
- Symptom: running the CLI with `--config` loaded YAML successfully for translation options, but provider credentials/endpoints/model/temperature were still ignored by `build_backend`, causing runtime failures unless the same values were repeated via CLI or env
- Root cause: config merging only produced `TranslateOptions`; backend construction continued reading raw clap/env fields directly, and clap defaulted `model`/`temperature` in a way that masked whether the user had explicitly overridden them
- Resolution: introduced `Config::resolve_provider_config(&Cli)` for backend-facing settings, taught the CLI to recognize default model/temperature values as fallback defaults, and routed shipped backend construction through the resolved config
- Prevention: keep backend-construction precedence in one tested helper and add regression tests that assert YAML-backed provider construction works without redundant CLI flags
- Related files: `src/cli.rs`, `src/config.rs`, `src/provider/mod.rs`, `src/provider/openai_compat.rs`, `src/provider/deepl.rs`, `src/provider/deeplx.rs`

## DeepLX marker-based batching destroyed by translation engine

- Date: 2026-04-22
- Area: provider / deeplx
- Symptom: all DeepLX translations failed with `DeepLX batch response missing marker __MDT_SEGMENT_1__`
- Root cause: the marker-based batching approach concatenated multiple text segments into a single string with `__MDT_SEGMENT_N__` delimiters, but DeepL's translation engine treated the markers as translatable text and translated, mangled, or stripped them — making response unpacking impossible
- Resolution: replaced marker-based batching with per-item requests (one HTTP request per `BatchItem`); DeepLX only accepts a single `text` string per request and has no native array batching, so per-item is the only reliable approach
- Prevention: do not embed structural markers into text sent to translation APIs that lack native batch support — the translation engine will destroy them; only use native batching (like DeepL's `text` array) or structured formats (like XML with LLM providers that understand XML)
- Related files: `src/provider/deeplx.rs`, `tests/provider_smoke.rs`

## Fixed concurrency over-drove unknown LLM/provider limits

- Date: 2026-04-22
- Area: engine / provider / config
- Symptom: translation throughput looked acceptable for some providers but collapsed unpredictably on others with 429s, timeouts, or rising tail latency because one static concurrency value was reused across different endpoints and runtime conditions
- Root cause: the runtime used a fixed semaphore limit for the full document, while provider HTTP responses discarded `Retry-After` hints and the retry loop had no way to reduce concurrency after rate limiting or overload
- Resolution: added adaptive runtime concurrency controls, structured provider HTTP errors with parsed `Retry-After`, and wave-based scheduling that increases on stable low-latency successes and decreases on 429s, timeouts, and elevated latency
- Prevention: for provider-facing async pipelines, persist upper-bound concurrency as configuration but let the active run adapt based on observed feedback instead of assuming one static safe value
- Related files: `src/engine.rs`, `src/error.rs`, `src/provider/mod.rs`, `src/provider/openai_compat.rs`, `src/provider/deepl.rs`, `src/provider/deeplx.rs`, `src/provider/gtx.rs`, `src/config.rs`, `providers.yaml.example`, `README.md`

## Cargo prerelease dependency and disabled disk tier broke Rust 1.95 verification

- Date: 2026-04-27
- Area: cargo / cache
- Symptom: `cargo clippy --workspace --all-targets --all-features -- -D warnings` failed before checking code because `sled = "1.0"` does not match `1.0.0-alpha.*`; after fixing dependency resolution, the two-tier cache persistence test failed because the disk tier was disabled
- Root cause: Cargo requires prerelease dependencies to be requested explicitly, while `Cargo.lock` already contained `sled 1.0.0-alpha.124`; separately, `TwoTierCache` had drifted from its documented memory + disk behavior and only stored values in memory
- Resolution: pinned `sled` to `1.0.0-alpha.124`, implemented `Cache` for `MemoryCache`, and restored `TwoTierCache` disk lookup/write-through with memory promotion and memory-only fallback when disk initialization fails
- Prevention: keep manifest prerelease versions exact, run Clippy from a clean dependency resolution state, and keep cache persistence tests aligned with the documented cache layers
- Related files: `Cargo.toml`, `src/cache/memory.rs`, `src/cache/two_tier.rs`, `src/cache/mod.rs`
