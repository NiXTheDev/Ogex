# Ogex

A custom regex engine with unified syntax for named groups and backreferences.

## Overview

Ogex introduces a cleaner, more intuitive syntax for named capturing groups and backreferences:

- **Named groups**: `(name:pattern)` instead of `(?<name>pattern)`
- **Backreferences**: `\g{name}` or `\g{1}` instead of `\k<name>` or `\1`
- **Works identically** in patterns and replacement strings

The engine is written in Rust for performance and provides bindings for multiple languages.

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

# Convert syntax
ogex convert "(name:abc)"
# Output: (?<name>abc)
```

## License

- ogex: MPL-2.0
- ogex-cli: MIT
