# ogex-cli

CLI tool for the Ogex regex engine.

## Installation

```bash
cargo install ogex-cli
```

## Usage

```bash
# Test a pattern against input
ogex test "(name:hello)" "hello world"

# Find all matches
ogex find "a+" "banana"

# Check if pattern matches
ogex match "abc" "abcdef"

# Convert Ogex syntax to traditional regex
ogex convert "(name:abc)"
# Output: (?<name>abc)

# Replace matches
ogex replace "(name:\w+)" "[$1]" "hello name:world"
# Output: hello [world]
```

## License

MIT
