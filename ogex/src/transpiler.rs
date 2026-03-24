//! Transpiler for converting regex syntax between flavors
//!
//! This module provides functionality to convert patterns between:
//! - Ogex syntax: `(name:pattern)`
//! - PCRE/.NET syntax: `(?<name>pattern)`
//! - Python syntax: `(?P<name>pattern)`

use crate::error::Result;
use crate::parser::parse;

/// Transpile Ogex to legacy/PCRE syntax
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
    Ok(ast.to_pcre_string())
}

/// Transpile Ogex to Python syntax
pub fn transpile_to_python(input: &str) -> Result<String> {
    let ast = parse(input)?;
    Ok(ast.to_python_string())
}

/// Transpile Ogex to Ogex format (for validation)
pub fn transpile_to_ogex(input: &str) -> Result<String> {
    let ast = parse(input)?;
    Ok(ast.to_ogex_string())
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

/// Convert a pattern to all supported flavors
pub fn convert_all(input: &str) -> Result<ConvertResult> {
    let ast = parse(input)?;

    Ok(ConvertResult {
        input: input.to_string(),
        ogex: ast.to_ogex_string(),
        python: ast.to_python_string(),
        pcre: ast.to_pcre_string(),
    })
}

/// Result of converting to all flavors
#[derive(Debug, Clone)]
pub struct ConvertResult {
    /// The original input pattern
    pub input: String,
    /// Ogex format: (name:pattern)
    pub ogex: String,
    /// Python format: (?P<name>pattern)
    pub python: String,
    /// PCRE format: (?<name>pattern)
    pub pcre: String,
}

impl ConvertResult {
    /// Print a formatted report of all conversions
    pub fn report(&self) {
        println!("Conversion Result");
        println!("================");
        println!("Input:  {}", self.input);
        println!();
        println!("Ogex:   {}", self.ogex);
        println!("Python: {}", self.python);
        println!("PCRE:   {}", self.pcre);
    }
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

/// Explain a regex pattern in human-readable format
pub fn explain(input: &str) -> Result<ExplainResult> {
    let ast = parse(input)?;

    // Generate various representations
    let ogex = ast.to_ogex_string();
    let python = ast.to_python_string();
    let pcre = ast.to_pcre_string();

    // Generate description
    let description = describe_ast(&ast);

    Ok(ExplainResult {
        input: input.to_string(),
        ogex,
        python,
        pcre,
        description,
    })
}

/// Result of explaining a regex pattern
#[derive(Debug, Clone)]
pub struct ExplainResult {
    /// The original input pattern
    pub input: String,
    /// Ogex format
    pub ogex: String,
    /// Python format  
    pub python: String,
    /// PCRE format
    pub pcre: String,
    /// Human-readable description
    pub description: String,
}

impl ExplainResult {
    /// Print a formatted explanation
    pub fn explain(&self) {
        println!("Regex Pattern Explanation");
        println!("========================");
        println!();
        println!("Input:  {}", self.input);
        println!();
        println!("Syntax Conversions:");
        println!("  Ogex:    {}", self.ogex);
        println!("  Python:  {}", self.python);
        println!("  PCRE:    {}", self.pcre);
        println!();
        println!("Description:");
        for line in self.description.lines() {
            println!("  {}", line);
        }
    }
}

/// Describe an AST in human-readable format
fn describe_ast(ast: &crate::ast::Expr) -> String {
    let mut description = String::new();
    describe_expr(ast, &mut description, 0);
    description.trim().to_string()
}

fn describe_expr(expr: &crate::ast::Expr, desc: &mut String, indent: usize) {
    let prefix = "  ".repeat(indent);
    match expr {
        crate::ast::Expr::Empty => {
            desc.push_str(&format!("{}Empty pattern\n", prefix));
        }
        crate::ast::Expr::Literal(c) => {
            desc.push_str(&format!("{}Match literal '{}'\n", prefix, c));
        }
        crate::ast::Expr::Any => {
            desc.push_str(&format!("{}Match any single character\n", prefix));
        }
        crate::ast::Expr::CharacterClass(class) => {
            let negated = if class.negated { "NOT " } else { "" };
            desc.push_str(&format!(
                "{}Match any character {}in: [{:?}]\n",
                prefix, negated, class
            ));
        }
        crate::ast::Expr::StartAnchor => {
            desc.push_str(&format!("{}Match at start of string\n", prefix));
        }
        crate::ast::Expr::EndAnchor => {
            desc.push_str(&format!("{}Match at end of string\n", prefix));
        }
        crate::ast::Expr::WordBoundary => {
            desc.push_str(&format!("{}Match at word boundary\n", prefix));
        }
        crate::ast::Expr::NonWordBoundary => {
            desc.push_str(&format!("{}Match at non-word boundary\n", prefix));
        }
        crate::ast::Expr::Group(inner) => {
            desc.push_str(&format!("{}Capturing group:\n", prefix));
            describe_expr(inner, desc, indent + 1);
        }
        crate::ast::Expr::NamedGroup { name, pattern } => {
            desc.push_str(&format!("{}Named capturing group '{}':\n", prefix, name));
            describe_expr(pattern, desc, indent + 1);
        }
        crate::ast::Expr::NonCapturingGroup(inner) => {
            desc.push_str(&format!("{}Non-capturing group:\n", prefix));
            describe_expr(inner, desc, indent + 1);
        }
        crate::ast::Expr::Lookahead(inner) => {
            desc.push_str(&format!(
                "{}Positive lookahead - must be followed by:\n",
                prefix
            ));
            describe_expr(inner, desc, indent + 1);
        }
        crate::ast::Expr::NegativeLookahead(inner) => {
            desc.push_str(&format!(
                "{}Negative lookahead - must NOT be followed by:\n",
                prefix
            ));
            describe_expr(inner, desc, indent + 1);
        }
        crate::ast::Expr::Lookbehind(inner) => {
            desc.push_str(&format!(
                "{}Positive lookbehind - must be preceded by:\n",
                prefix
            ));
            describe_expr(inner, desc, indent + 1);
        }
        crate::ast::Expr::NegativeLookbehind(inner) => {
            desc.push_str(&format!(
                "{}Negative lookbehind - must NOT be preceded by:\n",
                prefix
            ));
            describe_expr(inner, desc, indent + 1);
        }
        crate::ast::Expr::AtomicGroup(inner) => {
            desc.push_str(&format!("{}Atomic group (no backtracking):\n", prefix));
            describe_expr(inner, desc, indent + 1);
        }
        crate::ast::Expr::ConditionalGroup(inner) => {
            desc.push_str(&format!("{}Conditional group:\n", prefix));
            describe_expr(inner, desc, indent + 1);
        }
        crate::ast::Expr::ModeFlagsGroup { flags, pattern } => {
            desc.push_str(&format!("{}Mode flags ({}) applied to:\n", prefix, flags));
            describe_expr(pattern, desc, indent + 1);
        }
        crate::ast::Expr::Backreference(n) => {
            desc.push_str(&format!("{}Backreference to group {}\n", prefix, n));
        }
        crate::ast::Expr::RelativeBackreference(n) => {
            desc.push_str(&format!(
                "{}Relative backreference to group {} from end\n",
                prefix, n
            ));
        }
        crate::ast::Expr::NamedBackreference(name) => {
            desc.push_str(&format!(
                "{}Backreference to named group '{}'\n",
                prefix, name
            ));
        }
        crate::ast::Expr::Shorthand(c) => {
            let descr = match c {
                'd' => "digit (0-9)",
                'D' => "non-digit",
                'w' => "word character (a-z, A-Z, 0-9, _)",
                'W' => "non-word character",
                's' => "whitespace",
                'S' => "non-whitespace",
                _ => "unknown",
            };
            desc.push_str(&format!("{}Match {}\n", prefix, descr));
        }
        crate::ast::Expr::Sequence(exprs) => {
            for expr in exprs {
                describe_expr(expr, desc, indent);
            }
        }
        crate::ast::Expr::Alternation(exprs) => {
            desc.push_str(&format!(
                "{}Match ANY of {} alternatives:\n",
                prefix,
                exprs.len()
            ));
            for (i, expr) in exprs.iter().enumerate() {
                desc.push_str(&format!("{}  {}. ", prefix, i + 1));
                match expr {
                    crate::ast::Expr::Literal(c) => {
                        desc.push_str(&format!("'{}'\n", c));
                    }
                    _ => {
                        desc.push('\n');
                        describe_expr(expr, desc, indent + 2);
                    }
                }
            }
        }
        crate::ast::Expr::Quantified {
            quantifier,
            expr,
            greedy,
        } => {
            let (quantifier_str, descr): (String, String) = match quantifier {
                crate::ast::Quantifier::Optional => ("?".to_string(), "zero or one".to_string()),
                crate::ast::Quantifier::ZeroOrMore => ("*".to_string(), "zero or more".to_string()),
                crate::ast::Quantifier::OneOrMore => ("+".to_string(), "one or more".to_string()),
                crate::ast::Quantifier::Exactly(n) => {
                    (format!("{{{}}}", n), format!("exactly {}", n))
                }
                crate::ast::Quantifier::AtLeast(n) => {
                    (format!("{{{},}}", n), format!("at least {}", n))
                }
                crate::ast::Quantifier::Between(n, m) => {
                    (format!("{{{},{}}}", n, m), format!("{} to {}", n, m))
                }
            };
            // Add lazy suffix if not greedy
            let quantifier_str = if *greedy {
                quantifier_str
            } else {
                format!("{}?", quantifier_str)
            };
            let descr = if *greedy {
                descr
            } else {
                format!("{} (lazy)", descr)
            };
            let is_group = matches!(**expr, crate::ast::Expr::Group(_));
            let prefix_str = if is_group { "" } else { "Repeat: " };
            desc.push_str(&format!(
                "{}{}{} - {}\n",
                prefix, prefix_str, quantifier_str, descr
            ));
            describe_expr(expr, desc, indent + 1);
        }
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
