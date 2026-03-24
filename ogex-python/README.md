# Ogex Python Bindings

Python bindings for the Ogex regex engine with unified syntax support.

## Installation

```bash
pip install ogex
```

## Usage

```python
import ogex

# Basic matching
regex = ogex.Regex(r"hello (\w+)")
match = regex.search("hello world")
if match:
    print(match.text)  # "hello world"
    print(match.group(1))  # "world"

# Named groups with unified syntax
regex = ogex.Regex(r"(name:\w+) is \g{name}")
match = regex.search("John is John")
if match:
    print(match.text)  # "John is John"
    print(match.named_group("name"))  # "John"

# Relative backreferences
regex = ogex.Regex(r"(a)(b)\g{-1}")
print(regex.is_match("abb"))  # True

# Replacements
result = ogex.sub(r"(name:\w+)", r"[\g{name}]", "hello name:world")
print(result)  # "hello [world]"

# Find all matches
regex = ogex.Regex(r"\w+")
matches = regex.findall("hello world")
print([m.text for m in matches])  # ["hello", "world"]
```

## API

### Regex(pattern)
Compile a regex pattern.

### regex.search(string)
Search for the first match.

### regex.match_(string)
Match at the start of the string.

### regex.is_match(string)
Check if pattern matches.

### regex.findall(string)
Find all matches.

### regex.sub(repl, string, count=None)
Replace matches.

## License

MPL-2.0
