# Ogex Python Bindings

Python bindings for the Ogex regex engine with unified syntax support.

## Installation

```bash
pip install ogex
```

## Usage

```python
import ogex

# Compile a pattern
regex = ogex.compile("(name:\\w+)")

# Search for a match
match = regex.search("hello name:John")
if match:
    print(match.text)  # "name:John"
    print(match.group(1))  # "John"

# Relative backreferences
regex = ogex.compile("(a)(b)\\g{-1}")
print(regex.is_match("abb"))  # True
```
