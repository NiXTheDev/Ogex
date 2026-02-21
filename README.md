# Ogex

A custom regex engine with unified syntax for named groups and backreferences.

## Overview

Ogex is a regex engine that introduces a cleaner, more intuitive syntax for named capturing groups and backreferences:

- **Named groups**: `(name:pattern)` instead of `(?<name>pattern)`
- **Backreferences**: `\g{name}` or `\g{1}` instead of `\k<name>` or `\1`
- **Works identically** in patterns and replacement strings

The engine is written in Rust for performance and provides bindings for multiple languages via C FFI and WebAssembly.

## Features

- ✅ **Unified Syntax**: `(name:pattern)` for named groups, `\g{name}` for backreferences
- ✅ **Full Regex Support**: Quantifiers, alternation, character classes, anchors, groups
- ✅ **Backreferences**: Numbered (`\1`, `\2`), named (`\g{name}`), and relative (`\g{-1}`)
- ✅ **Entire Match**: `\G` for entire match in replacements
- ✅ **Multiple Bindings**: Rust library, C FFI, WebAssembly for JavaScript
- ✅ **CLI Tool**: `ogex` command for testing and conversion
- ✅ **Zero Warnings**: Clean, well-tested codebase

## Syntax Comparison

| Feature | Ogex | Traditional |
|---------|----------|-------------|
| Named group | `(name:abc)` | `(?<name>abc)` or `(?P<name>abc)` |
| Named backref | `\g{name}` | `\k<name>` or `\k'name'` |
| Numbered backref | `\g{1}` | `\1` |
| Relative backref | `\g{-1}` | Not supported |
| Entire match (replacement) | `\G` | `$&` or `\0` |

### Relative Backreferences (New!)

Ogex supports **relative backreferences** that reference numbered groups from the end:

| Syntax | Meaning |
|--------|---------|
| `\g{-1}` | Last numbered capture group |
| `\g{-2}` | Second-to-last numbered capture group |
| `\g{-n}` | nth numbered group from the end |

**Important:** Relative backreferences only count **numbered (non-named)** groups.

```rust
// Example: (a)(b)(c)\g{-1} matches "abcc"
// Numbered groups: 1=a, 2=b, 3=c
// \g{-1} references group 3 (last numbered)
```

**With named groups:**
```rust
// Pattern: (name:x)(a)(b)\g{-1}
// Numbered groups only: 2=a, 3=b (group 1 is named, excluded)
// \g{-1} references group 3 (last numbered = "b")
```

### Entire Match Reference (New!)

In replacement strings, use `\G` to reference the entire match:

```rust
// Wrap matches in brackets
let repl = Replacement::parse(r"[\G]");
// "hello" → "[hello]"
```

## Quick Start

### Rust

```rust
use ogex::Regex;

let regex = Regex::new(r"(name:\w+) is \g{name}").unwrap();
let m = regex.find("John is John").unwrap();
assert_eq!(m.text(), "John is John");
```

### CLI

```bash
# Test a pattern
ogex test "(name:hello)" "hello world"

# Convert to legacy syntax
ogex convert "(name:abc)"
# Output: (?<name>abc)

# Find all matches
ogex find "a+" "banana"

# Check if matches
ogex match "abc" "abc" && echo "yes"
```

### JavaScript (WASM)

```javascript
import init, { Regex } from './ogex.js';

await init();

const regex = new Regex("(name:hello)");
const m = regex.find("hello world");
console.log(m.text);  // "hello"
```

### C

```c
void* regex = ogex_compile("(name:hello)", NULL);
int matched = ogex_is_match(regex, "hello world");
ogex_free_regex(regex);
```

## Supported Syntax

### Literals
- `abc` - Match literal characters

### Character Classes
- `[abc]` - Match a, b, or c
- `[^abc]` - Match any character except a, b, or c
- `[a-z]` - Match range a through z
- `.` - Match any character

### Quantifiers
- `*` - Zero or more
- `+` - One or more
- `?` - Zero or one
- `{n}` - Exactly n times
- `{n,}` - At least n times
- `{n,m}` - Between n and m times

### Groups
- `(name:pattern)` - Named capturing group
- `(?:pattern)` - Non-capturing group
- `(pattern)` - Capturing group

### Anchors
- `^` - Start of string
- `$` - End of string

### Alternation
- `a|b|c` - Match a, b, or c

### Backreferences
- `\1`, `\2` - Numbered backreferences
- `\g{name}` - Named backreference
- `\g{1}` - Numbered backreference (alternative syntax)
- `\g{-1}`, `\g{-2}` - Relative backreferences (numbered groups from end)
- `\G` - Entire match (replacement strings only)

## Project Structure

```
ogex/
├── ogex/      # Core regex library
│   ├── src/
│   │   ├── lexer.rs    # Tokenizer
│   │   ├── parser.rs   # Recursive descent parser
│   │   ├── ast.rs      # Abstract syntax tree
│   │   ├── nfa.rs      # NFA construction (Thompson's)
│   │   ├── engine.rs   # Matching engine
│   │   ├── groups.rs   # Group registry
│   │   ├── replace.rs  # Replacement engine
│   │   ├── ffi.rs      # C FFI bindings
│   │   └── wasm.rs     # WebAssembly bindings
│   └── Cargo.toml
├── ogex-cli/       # CLI tool
│   └── src/main.rs
├── Cargo.toml          # Workspace manifest
└── README.md
```

## Building

### Prerequisites
- Rust 1.70+ 
- Cargo

### Build Library

```bash
cargo build --release -p ogex
```

### Build CLI

```bash
cargo build --release -p ogex-cli
```

### Build WASM

```bash
# Install wasm-pack if not already installed
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build for bundler (webpack, etc.)
wasm-pack build ogex --target bundler --features wasm

# Build for Node.js
wasm-pack build ogex --target nodejs --features wasm
```

## Testing

```bash
# Run all tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture
```

## Architecture

1. **Lexer**: Tokenizes input into tokens (literals, parens, operators)
2. **Parser**: Builds AST using recursive descent
3. **NFA Construction**: Converts AST to NFA using Thompson's algorithm
4. **Matching**: Simulates NFA with epsilon closure
5. **Groups**: Tracks capture groups during matching

## Performance

The engine uses NFA simulation which provides:
- Linear time matching for most patterns
- Full backreference support (requires backtracking)
- Predictable performance characteristics

For extremely performance-critical applications, consider using a DFA-based engine for patterns without backreferences.

## Future Work

- [ ] Lookahead/lookbehind assertions
- [ ] Atomic groups
- [ ] Possessive quantifiers
- [ ] Unicode property classes
- [ ] Streaming/lazy matching
- [ ] More language bindings (Python, Ruby, etc.)

## License

(LICENSE)[./LICENSE]

## Contributing

Contributions welcome! Please ensure:
- All tests pass: `cargo test --workspace`
- No warnings: `cargo build` produces no warnings
- Code is formatted: `cargo fmt`
- Documentation is updated

## Why Ogex?

The name comes from **Cust**om R**egex** - a regex engine with a custom, unified syntax that's easier to read and write than traditional regex flavors.

The syntax `(name:pattern)` is more intuitive because:
- It looks like a function call: `name(argument)`
- The colon clearly separates name from pattern
- It's consistent with the backreference syntax `\g{name}`

## Acknowledgments

- Inspired by the need for cleaner regex syntax
- Built with Rust for safety and performance
- Uses Thompson's construction algorithm for NFA generation
