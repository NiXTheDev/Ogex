//! Error types for the regex engine
//!
//! This module provides comprehensive error handling.
//! Errors are categorized by their source: lexing, parsing, compilation, or runtime.

use std::fmt;

#[cfg(feature = "std")]
use thiserror::Error;

/// The main error type for the regex engine
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum RegexError {
    /// Errors that occur during lexing/tokenization
    #[cfg_attr(feature = "std", error("lexer error at position {position}: {kind}"))]
    Lexer {
        /// Position in the input where the error occurred
        position: usize,
        /// The specific kind of lexer error
        kind: LexerErrorKind,
    },

    /// Errors that occur during parsing
    #[cfg_attr(feature = "std", error("parse error: {0}"))]
    Parse(#[cfg_attr(feature = "std", from)] ParseError),

    /// Errors that occur during compilation (AST to NFA/DFA)
    #[cfg_attr(feature = "std", error("compilation error: {0}"))]
    Compile(String),

    /// Errors that occur during pattern matching
    #[cfg_attr(feature = "std", error("runtime error: {0}"))]
    Runtime(String),
}

impl Clone for RegexError {
    fn clone(&self) -> Self {
        match self {
            RegexError::Lexer { position, kind } => RegexError::Lexer {
                position: *position,
                kind: kind.clone(),
            },
            RegexError::Parse(err) => RegexError::Parse(err.clone()),
            RegexError::Compile(msg) => RegexError::Compile(msg.clone()),
            RegexError::Runtime(msg) => RegexError::Runtime(msg.clone()),
        }
    }
}

impl From<ParseError> for RegexError {
    fn from(err: ParseError) -> Self {
        RegexError::Parse(err)
    }
}

impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegexError::Lexer { position, kind } => {
                write!(f, "lexer error at position {}: {}", position, kind)
            }
            RegexError::Parse(err) => write!(f, "parse error: {}", err),
            RegexError::Compile(msg) => write!(f, "compilation error: {}", msg),
            RegexError::Runtime(msg) => write!(f, "runtime error: {}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RegexError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RegexError::Parse(err) => Some(err),
            _ => None,
        }
    }
}

/// Specific kinds of lexer errors
#[derive(Debug, Clone, PartialEq)]
pub enum LexerErrorKind {
    /// Encountered an unexpected character
    UnexpectedChar(char),

    /// Unclosed character class (e.g., `[abc` without `]`)
    UnclosedCharacterClass,

    /// Invalid escape sequence
    InvalidEscape(char),

    /// Unclosed group
    UnclosedGroup,

    /// Invalid group name
    InvalidGroupName(String),
}

impl fmt::Display for LexerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerErrorKind::UnexpectedChar(c) => {
                write!(f, "unexpected character '{}'", c)
            }
            LexerErrorKind::UnclosedCharacterClass => {
                write!(f, "unclosed character class")
            }
            LexerErrorKind::InvalidEscape(c) => {
                write!(f, "invalid escape sequence '\\{}'", c)
            }
            LexerErrorKind::UnclosedGroup => write!(f, "unclosed group"),
            LexerErrorKind::InvalidGroupName(name) => {
                write!(f, "invalid group name '{}'", name)
            }
        }
    }
}

/// Errors that occur during parsing
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Unexpected token encountered
    UnexpectedToken {
        /// What was expected
        expected: String,
        /// What was actually found
        found: String,
        /// Location in the source (optional)
        span: Option<Span>,
    },

    /// Unexpected end of input
    UnexpectedEof {
        /// Location in the source (optional)
        span: Option<Span>,
    },

    /// Duplicate group name
    DuplicateGroupName(String),

    /// Undefined backreference
    UndefinedBackreference(String),

    /// Invalid quantifier
    InvalidQuantifier(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken {
                expected, found, ..
            } => {
                write!(f, "expected {}, found {}", expected, found)
            }
            ParseError::UnexpectedEof { .. } => write!(f, "unexpected end of input"),
            ParseError::DuplicateGroupName(name) => {
                write!(f, "duplicate group name '{}'", name)
            }
            ParseError::UndefinedBackreference(name) => {
                write!(f, "undefined backreference '{}'", name)
            }
            ParseError::InvalidQuantifier(msg) => {
                write!(f, "invalid quantifier: {}", msg)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {}

/// A span representing a location in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Start position (inclusive)
    pub start: usize,
    /// End position (exclusive)
    pub end: usize,
}

impl Span {
    /// Create a new span
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    /// Create a span for a single character
    pub fn single(pos: usize) -> Self {
        Span {
            start: pos,
            end: pos + 1,
        }
    }

    /// Get the length of the span
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if the span is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// An error with associated source location
#[derive(Debug, Clone)]
pub struct SpannedError {
    /// The underlying error
    pub error: RegexError,
    /// The location in the source
    pub span: Span,
}

impl fmt::Display for SpannedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at position {:?}", self.error, self.span)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SpannedError {}

impl SpannedError {
    /// Create a new spanned error
    pub fn new(error: RegexError, span: Span) -> Self {
        SpannedError { error, span }
    }
}

/// Result type alias for regex operations
pub type Result<T> = std::result::Result<T, RegexError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_error_display() {
        let err = RegexError::Lexer {
            position: 5,
            kind: LexerErrorKind::UnexpectedChar('!'),
        };
        assert_eq!(
            err.to_string(),
            "lexer error at position 5: unexpected character '!'"
        );
    }

    #[test]
    fn test_parse_error_unexpected_token() {
        let err = ParseError::UnexpectedToken {
            expected: "`)`".to_string(),
            found: "EOF".to_string(),
            span: None,
        };
        assert_eq!(err.to_string(), "expected `)`, found EOF");
    }

    #[test]
    fn test_parse_error_duplicate_group() {
        let err = ParseError::DuplicateGroupName("name".to_string());
        assert_eq!(err.to_string(), "duplicate group name 'name'");
    }

    #[test]
    fn test_regex_error_from_parse_error() {
        let parse_err = ParseError::UnexpectedEof { span: None };
        let regex_err: RegexError = parse_err.into();
        assert_eq!(
            regex_err.to_string(),
            "parse error: unexpected end of input"
        );
    }

    #[test]
    fn test_span_creation() {
        let span = Span::new(10, 20);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
        assert_eq!(span.len(), 10);
    }

    #[test]
    fn test_span_single() {
        let span = Span::single(5);
        assert_eq!(span.start, 5);
        assert_eq!(span.end, 6);
        assert_eq!(span.len(), 1);
    }

    #[test]
    fn test_spanned_error() {
        let error = RegexError::Parse(ParseError::UnexpectedEof { span: None });
        let spanned = SpannedError::new(error, Span::single(42));
        assert!(spanned.to_string().contains("unexpected end of input"));
        assert!(spanned.to_string().contains("42"));
    }
}
