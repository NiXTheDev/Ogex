//! Abstract Syntax Tree (AST) for regex patterns
//!
//! This module defines the AST types that represent parsed regex patterns.
//! Supports full regex syntax including:
//! - Literals, character classes, wildcards
//! - Quantifiers (*, +, ?, {n,m})
//! - Groups (capturing, non-capturing, named)
//! - Alternation (|)
//! - Anchors (^, $)
//! - Backreferences

use std::fmt;

/// An expression in the AST
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Empty expression (matches empty string)
    Empty,

    /// A literal character
    Literal(char),

    /// Any character (dot)
    Any,

    /// A sequence of expressions (concatenation)
    Sequence(Vec<Expr>),

    /// Alternation (e.g., a|b|c)
    Alternation(Vec<Expr>),

    /// A character class [abc] or [^abc] or [a-z]
    CharacterClass(CharacterClass),

    /// Quantified expression (e.g., a*, a+, a?, a{3,5})
    Quantified {
        /// The expression being quantified
        expr: Box<Expr>,
        /// The quantifier
        quantifier: Quantifier,
    },

    /// A capturing group: (...)
    Group(Box<Expr>),

    /// A non-capturing group: (?:...)
    NonCapturingGroup(Box<Expr>),

    /// A named capturing group: (name:...)
    NamedGroup {
        /// The name of the group
        name: String,
        /// The pattern inside the group
        pattern: Box<Expr>,
    },

    /// Start of string anchor (^)
    StartAnchor,

    /// End of string anchor ($)
    EndAnchor,

    /// Backreference by number (\1, \2, etc.)
    Backreference(u32),

    /// Relative backreference by negative index (\g{-1}, \g{-2}, etc.)
    /// References numbered groups only, from the end: \g{-1} = last numbered group
    RelativeBackreference(i32),

    /// Backreference by name (\g{name})
    NamedBackreference(String),

    /// Character class shorthand (\w, \d, \s, \W, \D, \S)
    Shorthand(char),

    /// Word boundary assertion (\b)
    WordBoundary,

    /// Non-word boundary assertion (\B)
    NonWordBoundary,

    /// Positive lookahead assertion (@>:pattern)
    Lookahead(Box<Expr>),

    /// Negative lookahead assertion (@>~:pattern)
    NegativeLookahead(Box<Expr>),

    /// Positive lookbehind assertion (@<:pattern)
    Lookbehind(Box<Expr>),

    /// Negative lookbehind assertion (@<~:pattern)
    NegativeLookbehind(Box<Expr>),

    /// Atomic group (@*:pattern)
    AtomicGroup(Box<Expr>),

    /// Conditional group (@%:pattern)
    ConditionalGroup(Box<Expr>),
}

/// A character class `[abc]`, `[^abc]`, or `[a-z]`
#[derive(Debug, Clone, PartialEq)]
pub struct CharacterClass {
    /// Whether the class is negated [^...]
    pub negated: bool,
    /// The items in the class
    pub items: Vec<ClassItem>,
}

/// An item in a character class
#[derive(Debug, Clone, PartialEq)]
pub enum ClassItem {
    /// A single character
    Char(char),
    /// A character range (e.g., a-z)
    Range(char, char),
    /// A character class shorthand (\d, \w, \s, etc.)
    Shorthand(char),
}

/// A quantifier
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Quantifier {
    /// Zero or more (*)
    ZeroOrMore,
    /// Zero or more lazy (*?)
    ZeroOrMoreLazy,
    /// One or more (+)
    OneOrMore,
    /// One or more lazy (+?)
    OneOrMoreLazy,
    /// Zero or one (?)
    Optional,
    /// Exactly n times ({n})
    Exactly(u32),
    /// At least n times ({n,})
    AtLeast(u32),
    /// At least n times lazy ({n,}?)
    AtLeastLazy(u32),
    /// Between n and m times ({n,m})
    Between(u32, u32),
    /// Between n and m times lazy ({n,m}?)
    BetweenLazy(u32, u32),
}

impl Expr {
    /// Create an empty expression
    pub fn empty() -> Self {
        Expr::Empty
    }

    /// Create a literal expression
    pub fn literal(c: char) -> Self {
        Expr::Literal(c)
    }

    /// Create an Any expression (.)
    pub fn any() -> Self {
        Expr::Any
    }

    /// Create a sequence from a vector of expressions
    pub fn sequence(exprs: Vec<Expr>) -> Self {
        match exprs.len() {
            0 => Expr::Empty,
            1 => exprs.into_iter().next().unwrap(),
            _ => Expr::Sequence(exprs),
        }
    }

    /// Create an alternation from a vector of expressions
    pub fn alternation(exprs: Vec<Expr>) -> Self {
        match exprs.len() {
            0 => Expr::Empty,
            1 => exprs.into_iter().next().unwrap(),
            _ => Expr::Alternation(exprs),
        }
    }

    /// Create a character class
    pub fn char_class(negated: bool, items: Vec<ClassItem>) -> Self {
        Expr::CharacterClass(CharacterClass { negated, items })
    }

    /// Create a quantified expression
    pub fn quantified(expr: Expr, quantifier: Quantifier) -> Self {
        Expr::Quantified {
            expr: Box::new(expr),
            quantifier,
        }
    }

    /// Create a capturing group
    pub fn group(expr: Expr) -> Self {
        Expr::Group(Box::new(expr))
    }

    /// Create a non-capturing group
    pub fn non_capturing_group(expr: Expr) -> Self {
        Expr::NonCapturingGroup(Box::new(expr))
    }

    /// Create a named group expression
    pub fn named_group(name: impl Into<String>, pattern: Expr) -> Self {
        Expr::NamedGroup {
            name: name.into(),
            pattern: Box::new(pattern),
        }
    }

    /// Create a start anchor (^)
    pub fn start_anchor() -> Self {
        Expr::StartAnchor
    }

    /// Create an end anchor ($)
    pub fn end_anchor() -> Self {
        Expr::EndAnchor
    }

    /// Create a backreference by number
    pub fn backreference(n: u32) -> Self {
        Expr::Backreference(n)
    }

    /// Create a relative backreference (\g{-n})
    pub fn relative_backreference(n: i32) -> Self {
        Expr::RelativeBackreference(n)
    }

    /// Create a named backreference
    pub fn named_backreference(name: impl Into<String>) -> Self {
        Expr::NamedBackreference(name.into())
    }

    /// Convert the AST back to a string (for debugging/transpilation)
    pub fn to_regex_string(&self) -> String {
        match self {
            Expr::Empty => String::new(),
            Expr::Literal(c) => c.to_string(),
            Expr::Any => ".".to_string(),
            Expr::Sequence(exprs) => exprs.iter().map(|e| e.to_regex_string()).collect(),
            Expr::Alternation(exprs) => {
                let parts: Vec<_> = exprs.iter().map(|e| e.to_regex_string()).collect();
                parts.join("|")
            }
            Expr::CharacterClass(cc) => cc.to_regex_string(),
            Expr::Quantified { expr, quantifier } => {
                let needs_parens = matches!(expr.as_ref(), Expr::Alternation(_));
                let expr_str = if needs_parens {
                    format!("(?:{})", expr.to_regex_string())
                } else {
                    expr.to_regex_string()
                };
                format!("{}{}", expr_str, quantifier.to_regex_string())
            }
            Expr::Group(expr) => format!("({})", expr.to_regex_string()),
            Expr::NonCapturingGroup(expr) => format!("(?:{})", expr.to_regex_string()),
            Expr::NamedGroup { name, pattern } => {
                format!("(?<{}>{})", name, pattern.to_regex_string())
            }
            Expr::StartAnchor => "^".to_string(),
            Expr::EndAnchor => "$".to_string(),
            Expr::Backreference(n) => format!("\\{}", n),
            Expr::RelativeBackreference(n) => format!("\\g{{{}}}", n),
            Expr::NamedBackreference(name) => format!("\\g{{{}}}", name),
            Expr::Shorthand(c) => format!("\\{}", c),
            Expr::WordBoundary => "\\b".to_string(),
            Expr::NonWordBoundary => "\\B".to_string(),
            Expr::Lookahead(expr) => format!("(@>:{})", expr.to_regex_string()),
            Expr::NegativeLookahead(expr) => format!("(@>~:{})", expr.to_regex_string()),
            Expr::Lookbehind(expr) => format!("(@<:{})", expr.to_regex_string()),
            Expr::NegativeLookbehind(expr) => format!("(@<~:{})", expr.to_regex_string()),
            Expr::AtomicGroup(expr) => format!("(@*:{})", expr.to_regex_string()),
            Expr::ConditionalGroup(expr) => format!("(@%:{})", expr.to_regex_string()),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_regex_string())
    }
}

impl CharacterClass {
    /// Convert character class to regex string
    fn to_regex_string(&self) -> String {
        let mut result = String::new();
        result.push('[');
        if self.negated {
            result.push('^');
        }
        for item in &self.items {
            match item {
                ClassItem::Char(c) => result.push(*c),
                ClassItem::Range(start, end) => {
                    result.push(*start);
                    result.push('-');
                    result.push(*end);
                }
                ClassItem::Shorthand(c) => {
                    result.push('\\');
                    result.push(*c);
                }
            }
        }
        result.push(']');
        result
    }
}

impl Quantifier {
    /// Convert quantifier to regex string
    fn to_regex_string(&self) -> String {
        match self {
            Quantifier::ZeroOrMore => "*".to_string(),
            Quantifier::ZeroOrMoreLazy => "*?".to_string(),
            Quantifier::OneOrMore => "+".to_string(),
            Quantifier::OneOrMoreLazy => "+?".to_string(),
            Quantifier::Optional => "?".to_string(),
            Quantifier::Exactly(n) => format!("{{{}}}", n),
            Quantifier::AtLeast(n) => format!("{{{},}}", n),
            Quantifier::AtLeastLazy(n) => format!("{{{},}}?", n),
            Quantifier::Between(n, m) => format!("{{{},{}}}", n, m),
            Quantifier::BetweenLazy(n, m) => format!("{{{},{}}}?", n, m),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let expr = Expr::empty();
        assert_eq!(expr.to_regex_string(), "");
    }

    #[test]
    fn test_literal() {
        let expr = Expr::literal('a');
        assert_eq!(expr.to_regex_string(), "a");
    }

    #[test]
    fn test_any() {
        let expr = Expr::any();
        assert_eq!(expr.to_regex_string(), ".");
    }

    #[test]
    fn test_sequence() {
        let expr = Expr::sequence(vec![
            Expr::literal('a'),
            Expr::literal('b'),
            Expr::literal('c'),
        ]);
        assert_eq!(expr.to_regex_string(), "abc");
    }

    #[test]
    fn test_alternation() {
        let expr = Expr::alternation(vec![
            Expr::literal('a'),
            Expr::literal('b'),
            Expr::literal('c'),
        ]);
        assert_eq!(expr.to_regex_string(), "a|b|c");
    }

    #[test]
    fn test_character_class() {
        let expr = Expr::char_class(
            false,
            vec![
                ClassItem::Char('a'),
                ClassItem::Char('b'),
                ClassItem::Char('c'),
            ],
        );
        assert_eq!(expr.to_regex_string(), "[abc]");
    }

    #[test]
    fn test_character_class_negated() {
        let expr = Expr::char_class(true, vec![ClassItem::Char('a'), ClassItem::Char('b')]);
        assert_eq!(expr.to_regex_string(), "[^ab]");
    }

    #[test]
    fn test_character_class_range() {
        let expr = Expr::char_class(
            false,
            vec![ClassItem::Range('a', 'z'), ClassItem::Range('0', '9')],
        );
        assert_eq!(expr.to_regex_string(), "[a-z0-9]");
    }

    #[test]
    fn test_quantifier_zero_or_more() {
        let expr = Expr::quantified(Expr::literal('a'), Quantifier::ZeroOrMore);
        assert_eq!(expr.to_regex_string(), "a*");
    }

    #[test]
    fn test_quantifier_one_or_more() {
        let expr = Expr::quantified(Expr::literal('a'), Quantifier::OneOrMore);
        assert_eq!(expr.to_regex_string(), "a+");
    }

    #[test]
    fn test_quantifier_optional() {
        let expr = Expr::quantified(Expr::literal('a'), Quantifier::Optional);
        assert_eq!(expr.to_regex_string(), "a?");
    }

    #[test]
    fn test_group() {
        let expr = Expr::group(Expr::sequence(vec![Expr::literal('a'), Expr::literal('b')]));
        assert_eq!(expr.to_regex_string(), "(ab)");
    }

    #[test]
    fn test_non_capturing_group() {
        let expr = Expr::non_capturing_group(Expr::literal('a'));
        assert_eq!(expr.to_regex_string(), "(?:a)");
    }

    #[test]
    fn test_named_group() {
        let expr = Expr::named_group(
            "name",
            Expr::sequence(vec![Expr::literal('a'), Expr::literal('b')]),
        );
        assert_eq!(expr.to_regex_string(), "(?<name>ab)");
    }

    #[test]
    fn test_start_anchor() {
        let expr = Expr::start_anchor();
        assert_eq!(expr.to_regex_string(), "^");
    }

    #[test]
    fn test_end_anchor() {
        let expr = Expr::end_anchor();
        assert_eq!(expr.to_regex_string(), "$");
    }

    #[test]
    fn test_backreference() {
        let expr = Expr::backreference(1);
        assert_eq!(expr.to_regex_string(), "\\1");
    }

    #[test]
    fn test_relative_backreference() {
        let expr = Expr::relative_backreference(-1);
        assert_eq!(expr.to_regex_string(), "\\g{-1}");
    }

    #[test]
    fn test_named_backreference() {
        let expr = Expr::named_backreference("name");
        assert_eq!(expr.to_regex_string(), "\\g{name}");
    }

    #[test]
    fn test_lookahead() {
        let expr = Expr::Lookahead(Box::new(Expr::literal('a')));
        assert_eq!(expr.to_regex_string(), "(@>:a)");
    }

    #[test]
    fn test_negative_lookahead() {
        let expr = Expr::NegativeLookahead(Box::new(Expr::literal('a')));
        assert_eq!(expr.to_regex_string(), "(@>~:a)");
    }

    #[test]
    fn test_lookbehind() {
        let expr = Expr::Lookbehind(Box::new(Expr::literal('a')));
        assert_eq!(expr.to_regex_string(), "(@<:a)");
    }

    #[test]
    fn test_negative_lookbehind() {
        let expr = Expr::NegativeLookbehind(Box::new(Expr::literal('a')));
        assert_eq!(expr.to_regex_string(), "(@<~:a)");
    }

    #[test]
    fn test_atomic_group() {
        let expr = Expr::AtomicGroup(Box::new(Expr::literal('a')));
        assert_eq!(expr.to_regex_string(), "(@*:a)");
    }

    #[test]
    fn test_conditional_group() {
        let expr = Expr::ConditionalGroup(Box::new(Expr::literal('a')));
        assert_eq!(expr.to_regex_string(), "(@%:a)");
    }
}
