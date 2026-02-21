//! Transpiler for converting custom regex syntax to legacy syntax
//!
//! This module provides functionality to convert patterns using our custom
//! syntax `(name:pattern)` to the legacy syntax `(?<name>pattern)`.

use crate::error::Result;
use crate::parser::parse;

/// Transpile a custom regex pattern to legacy syntax
///
/// # Example
/// ```
/// use ogex::transpile;
///
/// let result = transpile("(name:abc)").unwrap();
/// assert_eq!(result, "(?<name>abc)");
/// ```
pub fn transpile(input: &str) -> Result<String> {
    let ast = parse(input)?;
    Ok(ast.to_string())
}

/// Transpile with verbose output for debugging
pub fn transpile_debug(input: &str) -> Result<TranspileResult> {
    let ast = parse(input)?;
    let output = ast.to_string();

    Ok(TranspileResult {
        input: input.to_string(),
        ast: format!("{:?}", ast),
        output,
    })
}

/// Result of a transpilation with debug information
#[derive(Debug, Clone)]
pub struct TranspileResult {
    /// The original input pattern
    pub input: String,
    /// The AST representation (debug format)
    pub ast: String,
    /// The transpiled output
    pub output: String,
}

impl TranspileResult {
    /// Print a formatted report of the transpilation
    pub fn report(&self) {
        println!("Transpilation Report");
        println!("====================");
        println!("Input:  {}", self.input);
        println!("AST:    {}", self.ast);
        println!("Output: {}", self.output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transpile_simple_named_group() {
        let result = transpile("(name:abc)").unwrap();
        assert_eq!(result, "(?<name>abc)");
    }

    #[test]
    fn test_transpile_literals() {
        let result = transpile("abc").unwrap();
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_transpile_empty() {
        let result = transpile("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_transpile_nested_groups() {
        let result = transpile("(outer:(inner:abc))").unwrap();
        assert_eq!(result, "(?<outer>(?<inner>abc))");
    }

    #[test]
    fn test_transpile_simple_parens() {
        let result = transpile("(abc)").unwrap();
        assert_eq!(result, "(abc)");
    }

    #[test]
    fn test_transpile_complex_pattern() {
        let result = transpile("(first:hello) (second:world)").unwrap();
        // Note: space is preserved as literal character
        assert_eq!(result, "(?<first>hello) (?<second>world)");
    }

    #[test]
    fn test_transpile_error() {
        let result = transpile("(name:abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_transpile_debug() {
        let result = transpile_debug("(name:abc)").unwrap();
        assert_eq!(result.input, "(name:abc)");
        assert_eq!(result.output, "(?<name>abc)");
    }
}
