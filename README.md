# Ogex

A custom regex engine with unified syntax for named groups and backreferences.

## Overview

Ogex introduces a cleaner, more intuitive syntax for named capturing groups and backreferences:

- **Named groups**: `(name:pattern)` instead of `(?<name>pattern)`
- **Backreferences**: `\g{name}` or `\g{1}` instead of `\k<name>` or `\1`
- **Works identically** in patterns and replacement strings

The engine is written in Rust for performance and provides bindings for multiple languages.

## Features

### Core Engine
- **Escape sequences**: `\n`, `\t`, `\r` produce actual characters
- **Named groups**: Fully populated in Match results
- **Backreferences**: Both numbered (`\1`) and named (`\g{name}`)
- **Replace API**: `replace()` and `replace_all()` methods
- **Split API**: `split()` method to split by matches
- **Fullmatch API**: `fullmatch()` for exact string matching

### Python Bindings
- **Full re compatibility**: `search()`, `match()`, `findall()`, `sub()`, `split()`, `fullmatch()`
- **Named groups**: `named_group()` method on Match objects
- **Flags support**: `IGNORECASE`, `MULTILINE`, `DOTALL`
- **Version attribute**: `ogex.__version__`

### WASM Bindings
- **Full API**: `isMatch()`, `find()`, `findAll()`, `sub()`, `replaceAll()`, `split()`, `fullmatch()`, `match()`
- **TypeScript support**: Full type definitions

### CLI
- **Replace command**: `ogex replace "pattern" "replacement" "input"`
- **Stdin support**: Read from stdin with `-`
- **JSON output**: `--json` flag for structured output
- **Flags**: `--ignore-case`, `--multiline`, `--dotall`

## Crates

| Crate | Description |
|-------|-------------|
| [ogex](./ogex) | Core regex library (Rust) |
| [ogex-cli](./ogex-cli) | CLI tool |
| [ogex-python](./ogex-python) | Python bindings |

## Quick Start

### Rust

See [ogex/README.md](./ogex/README.md)

### JavaScript/WASM

```javascript
import { Regex } from '@ogex/ogex';

const regex = new Regex('(name:hello)');
const match = regex.find('hello world');
console.log(match.text);  // "hello"
```

### Python

```python
import ogex

regex = ogex.Regex(r"(name:\w+) is \g{name}")
match = regex.search("John is John")
print(match.named_group("name"))  # "John"
```

### CLI

```bash
# Test a pattern
ogex test "(name:hello)" "hello world"

# Replace
ogex replace "abc" "XYZ" "xxabcyy"

# Find with JSON output
ogex find "hello" "hello world" --json

# Read from stdin
echo "hello world" | ogex find "hello" -
```

## License

- ogex: MPL-2.0
- ogex-cli: MIT
