# ogex

A regex engine with unified syntax for named groups and backreferences.

## Features

- **Unified Syntax**: `(name:pattern)` for named groups, `\g{name}` for backreferences
- **Relative Backreferences**: `\g{-1}` references the last numbered group
- **Entire Match in Replacements**: `\G` for the entire match
- **Full Regex Support**: Quantifiers, alternation, character classes, anchors, groups
- **Multiple Targets**: Native Rust, WebAssembly, C FFI

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ogex = "0.1"
```

## Usage

```rust
use ogex::Regex;

// Basic matching
let regex = Regex::new(r"hello (\w+)").unwrap();
let m = regex.find("hello world").unwrap();
assert_eq!(m.group(1), Some("world"));

// Named groups with unified syntax
let regex = Regex::new(r"(name:\w+) is \g{name}").unwrap();
let m = regex.find("John is John").unwrap();
assert_eq!(m.named_group("name"), Some("John"));

// Relative backreferences
let regex = Regex::new(r"(a)(b)\g{-1}").unwrap();
assert!(regex.is_match("abb"));

// Replacements with entire match
use ogex::Replacement;
let repl = Replacement::parse(r"[\G]").unwrap();
let result = repl.apply("hello", &[]);
assert_eq!(result, "[hello]");
```

## Syntax

| Feature | Ogex | Traditional |
|---------|------|-------------|
| Named group | `(name:abc)` | `(?<name>abc)` |
| Named backref | `\g{name}` | `\k<name>` |
| Numbered backref | `\g{1}` | `\1` |
| Relative backref | `\g{-1}` | Not supported |
| Entire match (replacement) | `\G` | `$&` or `\0` |

## Feature Flags

- `wasm` - Enable WebAssembly bindings

## License

MPL-2.0
