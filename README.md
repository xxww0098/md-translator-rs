# md-translator-rs

`md-translator-rs` is a standalone Rust crate for translating Markdown or plain text while preserving document structure.

It can be used in two ways:

- as a CLI tool via the `mdxlate` binary
- as a reusable Rust library via `MdTranslator`

## Features

- **AST-driven translation** - parses Markdown via `comrak` into an abstract syntax tree, extracts translatable nodes (paragraphs, headings, list items, blockquotes, link text, image alt text, frontmatter values), translates them, and renders back to Markdown with 100% structure preservation
- **True batching** - DeepL providers batch up to 50 texts per request; AI providers pack segments into XML with `<seg id="N">` tags for efficient batch translation (default 15 segments per request, 90%+ request reduction)
- **Two-tier caching** - hot layer via `moka` (in-memory, TTL-based), cold layer via `sled` (persistent, embedded KV)
- **Request deduplication** - concurrent identical translation requests are coalesced automatically, saving 20-40% of API calls
- **Multi-provider fallback** - chain multiple providers; on retry-exhausted failure, automatically switches to the next provider for the full document
- **YAML configuration** - store provider credentials and performance settings in `providers.yaml`; CLI args override YAML values
- **Preserve Markdown structure** - frontmatter, fenced code blocks, inline code, LaTeX, links, headings, lists, blockquotes, HTML fragments are protected and not translated
- **Pluggable translation backends**
  - ordinary providers: `gtx`, `deep-lx`
  - AI providers: `openai-compat`, `deep-l`
- **Adaptive concurrency + retry pipeline** with configurable backoff and `Retry-After` awareness
- **Provider-side shared scheduler** (`src/provider/concurrent.rs`) — all four backends (GTX, DeepLX, DeepL, OpenAI) route through a shared scheduler that manages bounded in-flight work units; engine wave concurrency and provider-side scheduling operate as separate layers
- **CLI + reusable library API**

## Project Layout

This repository is a Rust workspace with a root crate plus a standalone caller project:

```text
md-translator-rs/
├── AGENTS.md
├── Cargo.toml
├── Cargo.lock
├── README.md
├── docs/
├── examples/
│   └── live-provider-smoke/ # Standalone Rust caller for real provider smoke runs
├── providers.yaml.example
├── src/
│   ├── main.rs              # CLI entrypoint
│   ├── lib.rs               # Library exports
│   ├── ast/                 # AST parsing, extraction, batching, rendering
│   │   ├── mod.rs
│   │   ├── parser.rs        # MarkdownAst wrapper around comrak
│   │   ├── extractor.rs     # TranslatableNode extraction
│   │   ├── batch.rs         # XML segment packing for AI providers
│   │   ├── parse_response.rs # XML response parsing
│   │   ├── render.rs        # AST node replacement + markdown render
│   │   └── pipeline.rs      # Full parse → extract → translate → render pipeline
│   ├── cache/               # Two-tier cache (memory moka + disk sled)
│   │   ├── mod.rs           # Cache trait + FileCache (deprecated)
│   │   ├── memory.rs       # moka-based MemoryCache
│   │   ├── disk.rs         # sled-based DiskCache
│   │   └── two_tier.rs     # TwoTierCache (lookup order: memory → disk → miss)
│   ├── provider/            # Backend implementations
│   ├── config.rs            # YAML config parsing
│   ├── cli.rs               # CLI argument parsing
│   ├── client.rs            # reqwest Client builder (tuned pool settings)
│   ├── dedup.rs             # Request deduplication layer
│   ├── fallback.rs          # MultiProviderBackend with fallback
│   ├── reporting.rs         # TranslationReporter and ReporterState
│   ├── engine.rs            # MdTranslator core pipeline
│   ├── markdown.rs          # Legacy placeholder protection/restore
│   ├── types.rs             # TranslateOptions, BatchItem, etc.
│   └── error.rs             # MdTranslatorError
└── tests/                   # Integration and mock tests
```

## CLI Overview

The current CLI takes these core parameters:

- `--input`: input file path
- `--target`: target language code
- `--provider-kind`: provider category, either `ordinary` or `ai`
- `--provider`: concrete provider implementation

### Provider Model

The CLI uses a two-level provider model:

| Provider kind | Providers | Notes |
|---|---|---|
| `ordinary` | `gtx`, `deep-lx` | Lower-friction providers, usually no LLM-style prompt/model settings |
| `ai` | `openai-compat`, `deep-l` | AI/API-driven providers that may require API keys or custom endpoints |

For YAML config, `openai-compat` uses the `llm:` section, DeepLX uses `deeplx:`, and DeepL uses `deepl:`.

### Basic Help

From the crate root:

```bash
cargo run --bin mdxlate -- --help
```

## CLI Usage

### 1. Translate Markdown with an ordinary provider (`gtx`)

```bash
cargo run --bin mdxlate -- \
  --input "/path/to/file.md" \
  --target zh \
  --provider-kind ordinary \
  --provider gtx
```

### 2. Translate Markdown with DeepLX

DeepLX requires an endpoint.

```bash
DEEPLX_ENDPOINT="http://127.0.0.1:1188/translate" \
cargo run --bin mdxlate -- \
  --input "/path/to/file.md" \
  --target zh \
  --provider-kind ordinary \
  --provider deep-lx
```

Or explicitly:

```bash
cargo run --bin mdxlate -- \
  --input "/path/to/file.md" \
  --target zh \
  --provider-kind ordinary \
  --provider deep-lx \
  --deeplx-endpoint "http://127.0.0.1:1188/translate"
```

### 3. Translate with an OpenAI-compatible provider

OpenAI-compatible translation requires an explicit endpoint from CLI args, env, or YAML config. When `--config` is provided, YAML credentials/settings are used unless CLI or env values override them.

```bash
OPENAI_API_KEY="your-key" \
OPENAI_COMPAT_ENDPOINT="https://api.openai.com/v1/chat/completions" \
cargo run --bin mdxlate -- \
  --input "/path/to/file.md" \
  --target zh \
  --provider-kind ai \
  --provider openai-compat \
  --model gpt-4o-mini
```

Or pass credentials directly:

```bash
cargo run --bin mdxlate -- \
  --input "/path/to/file.md" \
  --target zh \
  --provider-kind ai \
  --provider openai-compat \
  --api-key "your-key" \
  --openai-endpoint "https://api.openai.com/v1/chat/completions" \
  --model gpt-4o-mini \
  --temperature 0.2
```

### 4. Translate with DeepL

DeepL translation requires an explicit endpoint from CLI args, env, or YAML config. When `--config` is provided, YAML credentials/settings are used unless CLI or env values override them.

```bash
DEEPL_API_KEY="your-key" \
DEEPL_ENDPOINT="https://api-free.deepl.com/v2/translate" \
cargo run --bin mdxlate -- \
  --input "/path/to/file.md" \
  --target zh \
  --provider-kind ai \
  --provider deep-l
```

### 5. Explicit output path

If `--output` is omitted, the CLI writes to:

```text
<input-stem>.<target-language>.<extension>
```

Example:

```bash
cargo run --bin mdxlate -- \
  --input "/path/to/file.md" \
  --output "/path/to/output.custom.zh.md" \
  --target zh \
  --provider-kind ordinary \
  --provider gtx
```

### 6. Disable cache for first-run timing or clean benchmarking

```bash
cargo run --bin mdxlate -- \
  --input "/path/to/file.md" \
  --target zh \
  --provider-kind ordinary \
  --provider gtx \
  --no-cache
```

## YAML Configuration

Instead of passing provider credentials via CLI flags, you can create a `providers.yaml` file and use the `--config` flag.

### Configuration file search order

1. `--config <path>` (explicit path)
2. `~/.config/md-translator-rs/providers.yaml` (default location)

### Example providers.yaml

See `providers.yaml.example` in the repository root for a full template.

```yaml
# Example: DeepLX + OpenAI-compatible + DeepL setup

deeplx:
  deeplx_endpoint: "https://deeplx.example.com/translate"

llm:
  openai_endpoint: "https://api.example.com/v1/chat/completions"
  api_key: "sk-your-openai-compatible-key"
  model: "your-model-name"
  temperature: 0.2

deepl:
  deepl_endpoint: "https://api-free.deepl.com/v2/translate"
  api_key: "your-deepl-api-key"

performance:
  batch_size: 25
  max_chars_per_batch: 6000
  context_window: 100
  concurrency: 8
  adaptive_concurrency: true
  initial_concurrency: 2
  min_concurrency: 1
  retries: 5
  retry_backoff_ms: 500
  timeout_secs: 120

cache:
  enabled: true
  namespace: "default"

markdown:
  translate_frontmatter: false
  translate_multiline_code: false
  translate_latex: false
  translate_link_text: true
```

DeepL still uses `--provider-kind ai` on the CLI, but its YAML credentials live under `deepl:` rather than `llm:`.

### Using YAML config

```bash
# Use default config path (~/.config/md-translator-rs/providers.yaml)
cargo run --bin mdxlate -- \
  --provider-kind ai \
  --provider openai-compat \
  --input "/path/to/file.md" \
  --target zh

# Use explicit config path
cargo run --bin mdxlate -- \
  --config "/path/to/providers.yaml" \
  --provider-kind ordinary \
  --provider deep-lx \
  --input "/path/to/file.md" \
  --target zh
```

### CLI override behavior

CLI arguments always take precedence over YAML values. For example, if `providers.yaml` sets `batch_size: 25` but you pass `--batch-size 30` on the CLI, the CLI value (30) is used.

## Real Provider Smoke App

The repository includes a standalone Rust project at `examples/live-provider-smoke/` that depends on the root crate via `path = "../.."` and calls the library API directly. This replaces the old shell-script smoke runner.

It uses these defaults:

- input: recursively scan `test_input/` for `.md` files
- output directory: `test_output/`
- config lookup: repository-root `./providers.yaml` unless `--config` is given
- translation options: provider settings, batching, retry, cache, and markdown switches come from `providers.yaml`
- terminal summary: grouped by translated file, with a provider/time table for each file

### Smoke app help

```bash
./run_test.sh --help
```

### Run all configured providers

```bash
./run_test.sh
```

### Run selected providers only

```bash
./run_test.sh gtx

./run_test.sh deeplx openai-compat
```

### Use an explicit config path or custom input directory

```bash
./run_test.sh \
  --config "/path/to/providers.yaml" \
  --input "/path/to/input-dir-or-file" \
  --output-dir "/path/to/output" \
  deepl
```

## Common Options

### Format

```bash
--format markdown
--format plain-text
```

### Source language

```bash
--source auto
--source en
--source zh
```

### Markdown translation behavior

```bash
--translate-frontmatter
--translate-multiline-code
--translate-latex
--no-translate-link-text
```

### Runtime and batching

```bash
--batch-size 20
--max-chars-per-batch 5000
--context-window 50
--concurrency 6
--retries 3
--retry-backoff-ms 300
--timeout-secs 60
```

`concurrency` remains the upper bound. When adaptive concurrency is enabled in YAML, the runtime starts from a lower initial value, increases on stable low-latency successes, and decreases on 429s, timeouts, and elevated latency. If a provider returns `Retry-After`, the retry loop respects that hint before applying exponential backoff.

### Prompt customization for AI providers

```bash
--system-prompt "You are a professional translator."
--user-prompt "Translate from {source} to {target}:\n\n{text}"
```

## Library Usage

You can also use this crate directly from Rust code.

```rust
use std::sync::Arc;

use md_translator_rs::{GtxBackend, MdTranslator, TranslateOptions, TwoTierCache};

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let backend = Arc::new(GtxBackend::new(client));
    let cache = Arc::new(TwoTierCache::new(std::path::PathBuf::from("./.cache")).unwrap());
    let translator = MdTranslator::new(backend, cache);

    let options = TranslateOptions::default();
    let translated = translator
        .translate_markdown("# Hello world", &options)
        .await
        .unwrap();

    println!("{}", translated);
}
```

## Output Behavior

On success, the CLI prints a short summary like:

```text
Translated lines: 106
Cache hits: 0
Batches: 6
Written to: /path/to/output.md
```

## Verification

From the crate root:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo build --workspace
cargo doc --workspace
```

If you want to inspect the current CLI surface directly:

```bash
cargo run --bin mdxlate -- --help
```
