# Migration Guide: From Traditional Regex to Ogex

This guide helps you transition from traditional regex syntax (Python `re`, JavaScript `RegExp`, PCRE) to Ogex's unified syntax.

## Quick Reference

| Feature | Traditional | Ogex |
|---------|-------------|------|
| Named group | `(?<name>abc)` or `(?P<name>abc)` | `(name:abc)` |
| Named backref | `\k<name>` or `(?P=name)` | `\g{name}` |
| Numbered backref | `\1`, `\2` | `\1`, `\2` or `\g{1}`, `\g{2}` |
| Relative backref | Not supported | `\g{-1}`, `\g{-2}` |
| Entire match (repl) | `$&` or `\0` | `\G` |

## Named Groups

### Traditional Syntax
```python
# Python
pattern = r"(?P<username>\w+)@(?P<domain>\w+\.\w+)"
# or
pattern = r"(?<username>\w+)@(?<domain>\w+\.\w+)"
```

### Ogex Syntax
```python
# Python with ogex
pattern = r"(username:\w+)@(domain:\w+\.\w+)"
```

**Why the change?**
- `(name:pattern)` is more intuitive - looks like a function call
- Consistent with `\g{name}` backreference syntax
- Easier to read and write

## Backreferences

### Numbered Backreferences

Traditional and Ogex both support `\1`, `\2` syntax:

```python
# Both work
pattern = r"(\w+)\s+\1"  # Match repeated words
pattern = r"(\w+)\s+\g{1}"  # Ogex alternative syntax
```

### Named Backreferences

**Traditional:**
```python
# Python
pattern = r"(?P<name>\w+)\s+(?P=name)"
# JavaScript
pattern = r"(?<name>\w+)\s+\k<name>"
```

**Ogex:**
```python
pattern = r"(name:\w+)\s+\g{name}"
```

### Relative Backreferences (New!)

Ogex introduces relative backreferences that reference numbered groups from the end:

```python
# \g{-1} = last numbered group
pattern = r"(a)(b)\g{-1}"  # Matches "abb"

# \g{-2} = second-to-last numbered group  
pattern = r"(a)(b)(c)\g{-2}"  # Matches "abcb"
```

**Important:** Named groups are excluded from relative indexing:

```python
# Pattern: (a)(name:x)(b)\g{-2}
# Numbered groups: 1=a, 2=b (named group excluded)
# \g{-2} references group 1 = "a"
```

## Replacement Strings

### Traditional Syntax

```python
# Python
result = re.sub(r"(\w+)", r"\1-suffix", text)
result = re.sub(r"(?P<name>\w+)", r"\g<name>-suffix", text)
result = re.sub(r"\w+", r"$&-wrapped", text)  # Entire match
```

### Ogex Syntax

```python
# Python with ogex
result = regex.sub(r"\g{1}-suffix", text)  # Group 1
result = regex.sub(r"\g{name}-suffix", text)  # Named group
result = regex.sub(r"[\G]", text)  # Entire match wrapped in brackets
```

## Common Patterns Migration

### Email Pattern

**Traditional:**
```python
pattern = r"(?P<local>[a-zA-Z0-9._%+-]+)@(?P<domain>[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})"
```

**Ogex:**
```python
pattern = r"(local:[a-zA-Z0-9._%+-]+)@(domain:[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})"
```

### URL Pattern

**Traditional:**
```python
pattern = r"https?://(?P<host>[a-zA-Z0-9.-]+)(?P<path>/[^\s]*)?"
```

**Ogex:**
```python
# Note: :// conflicts with named group syntax
# Use \: to escape the colon
pattern = r"https?\://(host:[a-zA-Z0-9.-]+)(path:/[^\s]*)?"
```

### Paired Delimiters

**Traditional:**
```python
pattern = r"(['\"])(.*?)\1"  # Match quoted strings
```

**Ogex:**
```python
pattern = r"(['\"])(.*?)\g{1}"  # or \1 still works
```

## API Differences

### Python

```python
# Traditional re
import re
regex = re.compile(r"(?P<name>\w+)")
match = regex.search("hello")
if match:
    print(match.group("name"))

# Ogex
import ogex
regex = ogex.compile(r"(name:\w+)")
match = regex.search("hello")
if match:
    print(match.named_group("name"))  # or match.group_str("hello", 1)
```

### JavaScript (WASM)

```javascript
// Traditional
const regex = /(?<name>\w+)/;
const match = regex.exec("hello");
console.log(match.groups.name);

// Ogex
import init, { Regex } from 'ogex';
await init();
const regex = new Regex("(name:\\w+)");
const match = regex.find("hello");
console.log(match.named_group("name"));
```

## Limitations & Known Issues

1. **`://` in patterns**: The sequence `://` conflicts with named group syntax. Use `\://` to escape.

2. **`.` in character classes**: Some edge cases with `.` inside `[]` may need attention.

3. **Lookahead/lookbehind**: Not yet supported. Use alternation or other constructs.

4. **Named group extraction**: Currently returns `(start, end)` positions rather than direct string access.

## Migration Checklist

- [ ] Replace `(?<name>...)` or `(?P<name>...)` with `(name:...)`
- [ ] Replace `\k<name>` or `(?P=name)` with `\g{name}`
- [ ] Consider using `\g{-n}` for relative backreferences
- [ ] Replace `$&` with `\G` in replacement strings
- [ ] Escape `:` when it's not part of a named group (e.g., `://` → `\://`)
- [ ] Update API calls for group access (`named_group()` vs `group("name")`)

## Need Help?

- **Documentation**: https://github.com/NiXTheDev/Ogex
- **Issues**: https://github.com/NiXTheDev/Ogex/issues
- **Examples**: See `tests/` directory for comprehensive examples
