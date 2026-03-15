# @ogex/ogex

JavaScript/TypeScript bindings for the Ogex regex engine.

## Installation

```bash
npm install @ogex/ogex
```

## Usage

```javascript
import { Regex, Match, Error } from '@ogex/ogex';

// Basic matching
const regex = new Regex('hello (\\w+)');
const match = regex.find('hello world');
console.log(match.text);  // "hello world"
console.log(match.group(1));  // "world"

// Named groups with unified syntax
const regex = new Regex('(name:\\w+) is \\g{name}');
const match = regex.find('John is John');
console.log(match.text);  // "John is John"
console.log(match.namedGroup('name'));  // "John"

// Relative backreferences
const regex = new Regex('(a)(b)\\g{-1}');
console.log(regex.isMatch('abb'));  // true

// Find all matches
const regex = new Regex('\\w+');
const matches = regex.findAll('hello world');
console.log(matches.map(m => m.text));  // ["hello", "world"]
```

## API

### Regex(pattern)
Create a new regex pattern.

### regex.find(string)
Find the first match.

### regex.isMatch(string)
Check if pattern matches.

### regex.findAll(string)
Find all matches.

## Syntax

| Feature | Ogex | Traditional |
|---------|------|-------------|
| Named group | `(name:abc)` | `(?<name>abc)` |
| Named backref | `\g{name}` | `\k<name>` |
| Numbered backref | `\g{1}` | `\1` |

## License

MPL-2.0
