---
title: Test Document
description: A sample markdown for translation smoke testing
author: md-translator-rs
---

# Introduction

This is a **test document** designed to verify that [md-translator-rs](https://github.com/example/md-translator-rs) correctly translates Markdown while preserving its structure.

## Features

- **Bold text** and *italic text* are preserved
- [Hyperlinks](https://example.com) keep their URLs intact
- ![Image alt text](https://example.com/image.png) preserves both alt and src
- Inline `code spans` are never translated

## Code Blocks

```rust
fn main() {
    println!("Hello, world!");
    let x = 42;
}
```

```python
def greet(name: str) -> str:
    return f"Hello, {name}!"
```

## Math & LaTeX

Inline math: $E = mc^2$

Block math:

$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$

## Tables

| Feature | Status | Notes |
|---------|--------|-------|
| Headings | ✅ Supported | All levels h1-h6 |
| Lists | ✅ Supported | Ordered and unordered |
| Tables | ✅ Supported | With alignment |
| Code | ✅ Preserved | Never translated |

## Blockquotes

> Translation is not merely a matter of words.
> It is a matter of making intelligible a whole culture.
>
> — Anthony Burgess

## Nested Lists

1. First level item
   - Second level bullet
   - Another bullet
     1. Third level numbered
     2. Another numbered
2. Back to first level

## Mixed Content

Here is a paragraph with **bold**, *italic*, `code`, and a [link](https://example.com/path?query=value&foo=bar).

The following URL should not be translated: https://api.example.com/v2/translate

## Edge Cases

- Empty list item:
- Item with `inline code` and **bold** together
- Item with [link text](https://example.com) in the middle of text
- Very long paragraph that spans multiple lines to test how the translator handles wrapping and line breaks in the output. This paragraph intentionally contains enough text to exceed typical line length limits and ensure proper handling.

---

*End of test document.*
