//! Error types for the regex engine
//!
//! This module provides comprehensive error handling using the `thiserror` crate.
//! Errors are categorized by their source: lexing, parsing, compilation, or runtime.

use thiserror::Error;

/// The main error type for the regex engine
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RegexError {
    /// Errors that occur during lexing/tokenization
    #[error("lexer error at position {position}: {kind}")]
    Lexer {
        /// Position in the input where the error occurred
        position: usize,
        /// The specific kind of lexer error
        kind: LexerErrorKind,
    },

    /// Errors that occur during parsing
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),

    /// Errors that occur during compilation (AST to NFA/DFA)
    #[error("compilation error: {0}")]
    Compile(String),

    /// Errors that occur during pattern matching
    #[error("runtime error: {0}")]
    Runtime(String),
}

/// Specific kinds of lexer errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum LexerErrorKind {
    /// Encountered an unexpected character
    #[error("unexpected character '{0}'")]
    UnexpectedChar(char),

    /// Unclosed character class (e.g., `[abc` without `]`)
    #[error("unclosed character class")]
    UnclosedCharacterClass,

    /// Invalid escape sequence
    #[error("invalid escape sequence '\\{0}'")]
    InvalidEscape(char),

    /// Unclosed group
    #[error("unclosed group")]
    UnclosedGroup,

    /// Invalid group name
    #[error("invalid group name '{0}'")]
    InvalidGroupName(String),
}

/// Errors that occur during parsing
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Unexpected token encountered
    #[error("expected {expected}, found {found}")]
    UnexpectedToken {
        /// What was expected
        expected: String,
        /// What was actually found
        found: String,
    },

    /// Unexpected end of input
    #[error("unexpected end of input")]
    UnexpectedEof,

    /// Duplicate group name
    #[error("duplicate group name '{0}'")]
    DuplicateGroupName(String),

    /// Undefined backreference
    #[error("undefined backreference '{0}'")]
    UndefinedBackreference(String),

    /// Invalid quantifier
    #[error("invalid quantifier: {0}")]
    InvalidQuantifier(String),
}

/// A span representing a location in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl Default for Span {
    fn default() -> Self {
        Span { start: 0, end: 0 }
    }
}

/// An error with associated source location
#[derive(Error, Debug, Clone)]
#[error("{error} at position {span:?}")]
pub struct SpannedError {
    /// The underlying error
    pub error: RegexError,
    /// The location in the source
    pub span: Span,
}

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
        let parse_err = ParseError::UnexpectedEof;
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
        let error = RegexError::Parse(ParseError::UnexpectedEof);
        let spanned = SpannedError::new(error, Span::single(42));
        assert!(spanned.to_string().contains("unexpected end of input"));
        assert!(spanned.to_string().contains("42"));
    }
}
