//! Ogex Core Library
//!
//! A custom regex engine with unified syntax for named groups and backreferences.

pub mod ast;
pub mod engine;
pub mod error;
pub mod ffi;
pub mod groups;
pub mod lexer;
pub mod nfa;
pub mod parser;
pub mod replace;
pub mod transpiler;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use ast::Expr;
pub use engine::{Match, Regex};
pub use error::{LexerErrorKind, ParseError, RegexError, Result, Span, SpannedError};
pub use groups::{GroupCollector, GroupInfo, GroupRegistry, GroupRegistryError};
pub use lexer::{Lexer, Token};
pub use nfa::{Nfa, State, StateId, Transition};
pub use parser::{Parser, parse};
pub use replace::{Replacement, ReplacementError, ReplacementPart};
pub use transpiler::{TranspileResult, transpile, transpile_debug};

/// Compile a regex pattern with custom syntax
///
/// This is the main entry point for compiling patterns.
/// For now, it just transpiles to legacy syntax.
pub fn compile(input: &str) -> Result<String> {
    transpile(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_end_to_end() {
        // Test the full pipeline: string -> tokens -> AST -> legacy syntax
        let pattern = "(name:abc)";
        let result = compile(pattern).unwrap();
        assert_eq!(result, "(?<name>abc)");
    }

    #[test]
    fn test_round_trip_concept() {
        // Demonstrate the proof-of-concept: custom syntax can be converted
        // Note: using simple pattern since full parser isn't implemented yet
        let custom = "(username:abc)";
        let legacy = compile(custom).unwrap();
        assert_eq!(legacy, "(?<username>abc)");
    }
}
