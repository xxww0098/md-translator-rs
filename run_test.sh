#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 模拟前端传入的 md 文件夹路径与输出路径
INPUT_DIR="./test_input"
OUTPUT_DIR="./test_output"

ARGS=()
for arg in "$@"; do
    case "$arg" in
        -llm|--llm)
            ARGS+=("llm")
            ;;
        -deeplx|--deeplx)
            ARGS+=("deeplx")
            ;;
        -deepl|--deepl)
            ARGS+=("deepl")
            ;;
        -gtx|--gtx)
            ARGS+=("gtx")
            ;;
        --no_cache|--no-cache|-no_cache|-no-cache)
            ARGS+=("--no-cache")
            ;;
        *)
            ARGS+=("$arg")
            ;;
    esac
done

cargo run -p live-provider-smoke -- \
    --input "$INPUT_DIR" \
    --output-dir "$OUTPUT_DIR" \
    "${ARGS[@]+"${ARGS[@]}"}"
