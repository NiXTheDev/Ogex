//! Replacement string handling
//!
//! This module handles replacement strings that can contain backreferences
//! like \g{name} or \g{1}, as well as special references:
//! - `\G` for the entire match
//! - `\g{0}` for the entire match (deprecated, use `\G` instead)

use std::collections::HashMap;

/// A part of a replacement string
#[derive(Debug, Clone, PartialEq)]
pub enum ReplacementPart {
    /// Literal text
    Literal(String),
    /// Backreference by number (\1, \2, etc.)
    BackrefNumber(u32),
    /// Backreference by name (\g{name})
    BackrefName(String),
    /// Entire match (\g{0} or $&)
    EntireMatch,
}

/// A parsed replacement string
#[derive(Debug, Clone)]
pub struct Replacement {
    parts: Vec<ReplacementPart>,
}

impl Replacement {
    /// Parse a replacement string
    pub fn parse(input: &str) -> Result<Self, ReplacementError> {
        let mut parts = Vec::new();
        let mut chars = input.chars().peekable();
        let mut current_literal = String::new();

        while let Some(c) = chars.next() {
            if c == '\\' {
                // Check for backreference
                if let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() {
                        // \1, \2, etc.
                        if !current_literal.is_empty() {
                            parts.push(ReplacementPart::Literal(current_literal.clone()));
                            current_literal.clear();
                        }
                        chars.next(); // consume digit
                        let mut num = next.to_digit(10).unwrap();
                        // Read additional digits
                        while let Some(&c) = chars.peek() {
                            if c.is_ascii_digit() {
                                chars.next();
                                num = num * 10 + c.to_digit(10).unwrap();
                            } else {
                                break;
                            }
                        }
                        parts.push(ReplacementPart::BackrefNumber(num));
                    } else if next == 'g' {
                        // \g{name} or \g{1} or \g{0}
                        chars.next(); // consume 'g'
                        if chars.peek() == Some(&'{') {
                            chars.next(); // consume '{'
                            let name = Self::read_until(&mut chars, '}');
                            if chars.peek() == Some(&'}') {
                                chars.next(); // consume '}'
                            }
                            if !current_literal.is_empty() {
                                parts.push(ReplacementPart::Literal(current_literal.clone()));
                                current_literal.clear();
                            }
                            // Check if it's a number (\g{0}, \g{1}) or name
                            if name == "0" {
                                parts.push(ReplacementPart::EntireMatch);
                            } else if let Ok(num) = name.parse::<u32>() {
                                parts.push(ReplacementPart::BackrefNumber(num));
                            } else {
                                parts.push(ReplacementPart::BackrefName(name));
                            }
                        } else {
                            // Not a valid \g{...}, treat as literal
                            current_literal.push(c);
                            current_literal.push(next);
                        }
                    } else if next == 'G' {
                        // \G - entire match reference
                        chars.next(); // consume 'G'
                        if !current_literal.is_empty() {
                            parts.push(ReplacementPart::Literal(current_literal.clone()));
                            current_literal.clear();
                        }
                        parts.push(ReplacementPart::EntireMatch);
                    } else {
                        // Escaped character, add to literal
                        chars.next();
                        current_literal.push(next);
                    }
                } else {
                    // Trailing backslash
                    current_literal.push(c);
                }
            } else {
                current_literal.push(c);
            }
        }

        // Don't forget the last literal
        if !current_literal.is_empty() {
            parts.push(ReplacementPart::Literal(current_literal));
        }

        Ok(Replacement { parts })
    }

    /// Read characters until delimiter
    fn read_until(chars: &mut std::iter::Peekable<std::str::Chars>, delimiter: char) -> String {
        let mut result = String::new();
        while let Some(&c) = chars.peek() {
            if c == delimiter {
                break;
            }
            result.push(c);
            chars.next();
        }
        result
    }

    /// Apply the replacement to a match
    pub fn apply(
        &self,
        original: &str,
        match_start: usize,
        match_end: usize,
        groups: &[(usize, usize)],
    ) -> String {
        self.apply_with_names(original, match_start, match_end, groups, &HashMap::new())
    }

    /// Apply the replacement to a match with named group support
    ///
    /// # Arguments
    /// * `original` - The original input string
    /// * `match_start` - Start position of the match
    /// * `match_end` - End position of the match
    /// * `groups` - Numbered capture groups as (start, end) pairs (1-indexed)
    /// * `named_groups` - Map from group name to group index
    pub fn apply_with_names(
        &self,
        original: &str,
        match_start: usize,
        match_end: usize,
        groups: &[(usize, usize)],
        named_groups: &HashMap<String, u32>,
    ) -> String {
        let mut result = String::new();

        for part in &self.parts {
            match part {
                ReplacementPart::Literal(text) => result.push_str(text),
                ReplacementPart::BackrefNumber(n) => {
                    if *n == 0 {
                        // Entire match
                        result.push_str(&original[match_start..match_end]);
                    } else if let Some(&(start, end)) = groups.get((*n as usize).saturating_sub(1))
                    {
                        result.push_str(&original[start..end]);
                    }
                    // If group doesn't exist, replace with empty string
                }
                ReplacementPart::BackrefName(name) => {
                    // Look up the named group index and get the captured text
                    if let Some(&group_index) = named_groups.get(name)
                        && let Some(&(start, end)) =
                            groups.get((group_index as usize).saturating_sub(1))
                        {
                            result.push_str(&original[start..end]);
                        }
                    // If named group doesn't exist, replace with empty string
                }
                ReplacementPart::EntireMatch => {
                    result.push_str(&original[match_start..match_end]);
                }
            }
        }

        result
    }

    /// Get the parts of the replacement
    pub fn parts(&self) -> &[ReplacementPart] {
        &self.parts
    }
}

/// Errors that can occur during replacement parsing
#[derive(Debug, Clone, PartialEq)]
pub enum ReplacementError {
    /// Invalid backreference
    InvalidBackreference(String),
}

impl std::fmt::Display for ReplacementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplacementError::InvalidBackreference(s) => {
                write!(f, "invalid backreference: {}", s)
            }
        }
    }
}

impl std::error::Error for ReplacementError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_literal() {
        let repl = Replacement::parse("hello").unwrap();
        assert_eq!(repl.parts.len(), 1);
        assert!(matches!(&repl.parts[0], ReplacementPart::Literal(s) if s == "hello"));
    }

    #[test]
    fn test_parse_backref_number() {
        let repl = Replacement::parse("\\1").unwrap();
        assert_eq!(repl.parts.len(), 1);
        assert!(matches!(&repl.parts[0], ReplacementPart::BackrefNumber(1)));
    }

    #[test]
    fn test_parse_backref_name() {
        let repl = Replacement::parse("\\g{name}").unwrap();
        assert_eq!(repl.parts.len(), 1);
        assert!(matches!(&repl.parts[0], ReplacementPart::BackrefName(s) if s == "name"));
    }

    #[test]
    fn test_parse_entire_match() {
        let repl = Replacement::parse("\\g{0}").unwrap();
        assert_eq!(repl.parts.len(), 1);
        assert!(matches!(&repl.parts[0], ReplacementPart::EntireMatch));
    }

    #[test]
    fn test_parse_entire_match_g() {
        // \G is the preferred syntax for entire match
        let repl = Replacement::parse("\\G").unwrap();
        assert_eq!(repl.parts.len(), 1);
        assert!(matches!(&repl.parts[0], ReplacementPart::EntireMatch));
    }

    #[test]
    fn test_parse_mixed_with_g() {
        let repl = Replacement::parse("prefix\\Gsuffix").unwrap();
        assert_eq!(repl.parts.len(), 3);
        assert!(matches!(&repl.parts[0], ReplacementPart::Literal(s) if s == "prefix"));
        assert!(matches!(&repl.parts[1], ReplacementPart::EntireMatch));
        assert!(matches!(&repl.parts[2], ReplacementPart::Literal(s) if s == "suffix"));
    }

    #[test]
    fn test_parse_mixed() {
        let repl = Replacement::parse("prefix\\1suffix").unwrap();
        assert_eq!(repl.parts.len(), 3);
        assert!(matches!(&repl.parts[0], ReplacementPart::Literal(s) if s == "prefix"));
        assert!(matches!(&repl.parts[1], ReplacementPart::BackrefNumber(1)));
        assert!(matches!(&repl.parts[2], ReplacementPart::Literal(s) if s == "suffix"));
    }

    #[test]
    fn test_apply_literal() {
        let repl = Replacement::parse("replacement").unwrap();
        let result = repl.apply("original", 0, 8, &[]);
        assert_eq!(result, "replacement");
    }

    #[test]
    fn test_apply_backref() {
        let repl = Replacement::parse("\\1").unwrap();
        // Match "abc" at position 0-3, group 1 is "b" at 1-2
        let result = repl.apply("abc", 0, 3, &[(1, 2)]);
        assert_eq!(result, "b");
    }

    #[test]
    fn test_apply_entire_match() {
        let repl = Replacement::parse("\\g{0}").unwrap();
        let result = repl.apply("hello world", 0, 5, &[]);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_apply_entire_match_g() {
        // \G should work the same as \g{0}
        let repl = Replacement::parse("\\G").unwrap();
        let result = repl.apply("hello world", 0, 5, &[]);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_apply_entire_match_with_brackets() {
        let repl = Replacement::parse("[\\G]").unwrap();
        let result = repl.apply("hello world", 0, 5, &[]);
        assert_eq!(result, "[hello]");
    }

    #[test]
    fn test_apply_mixed() {
        let repl = Replacement::parse("[\\1]").unwrap();
        let result = repl.apply("abc", 0, 3, &[(1, 2)]);
        assert_eq!(result, "[b]");
    }

    #[test]
    fn test_apply_multiple_groups() {
        let repl = Replacement::parse(r"\2-\1").unwrap();
        // Groups: 1="a" (0,1), 2="b" (1,2)
        let result = repl.apply("ab", 0, 2, &[(0, 1), (1, 2)]);
        assert_eq!(result, "b-a");
    }

    #[test]
    fn test_escape_sequences() {
        // \\n becomes literal 'n', \\t becomes literal 't' (not newline/tab)
        let repl = Replacement::parse("\\n\\t").unwrap();
        assert_eq!(repl.parts.len(), 1);
        assert!(matches!(&repl.parts[0], ReplacementPart::Literal(s) if s == "nt"));
    }

    #[test]
    fn test_apply_named_backref() {
        let repl = Replacement::parse("\\g{name}").unwrap();

        // Create named groups mapping: "name" -> group 1
        let mut named = HashMap::new();
        named.insert("name".to_string(), 1);

        // Match "hello world", group 1 is "hello" at 0-5
        let result = repl.apply_with_names("hello world", 0, 11, &[(0, 5)], &named);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_apply_named_backref_with_literal() {
        let repl = Replacement::parse("result: \\g{name}!").unwrap();

        let mut named = HashMap::new();
        named.insert("name".to_string(), 1);

        // Match "hello", group 1 is "hello"
        let result = repl.apply_with_names("hello", 0, 5, &[(0, 5)], &named);
        assert_eq!(result, "result: hello!");
    }

    #[test]
    fn test_apply_mixed_named_and_numbered() {
        let repl = Replacement::parse(r"\g{name}-\1-\G").unwrap();

        let mut named = HashMap::new();
        named.insert("name".to_string(), 2); // "name" is group 2

        // Groups: 1="a" (0,1), 2="b" (1,2)
        // Match "ab" at 0-2
        let result = repl.apply_with_names("ab", 0, 2, &[(0, 1), (1, 2)], &named);
        assert_eq!(result, "b-a-ab");
    }

    #[test]
    fn test_apply_missing_named_backref() {
        let repl = Replacement::parse("\\g{missing}").unwrap();

        let named = HashMap::new(); // No named groups

        // Should replace with empty string
        let result = repl.apply_with_names("hello", 0, 5, &[], &named);
        assert_eq!(result, "");
    }
}
