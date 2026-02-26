//! Parser for regex patterns
//!
//! This module provides a recursive descent parser that converts
//! tokens into an Abstract Syntax Tree (AST).
//!
//! Grammar (in order of precedence, lowest to highest):
//!   regex     := alternation
//!   alternation := sequence ( '|' sequence )*
//!   sequence  := quantified+
//!   quantified := atom quantifier?
//!   quantifier := '*' | '+' | '?' | '{' number (',' number?)? '}' '?'?
//!   atom      := literal | anchor | group | char_class | '.' | backref | escape
//!   literal   := any char except specials
//!   anchor    := '^' | '$'
//!   group     := '(' group_inner ')'
//!   group_inner := named_group | non_capturing | capturing
//!   named_group := identifier ':' sequence
//!   non_capturing := '?:' sequence
//!   capturing := sequence
//!   char_class := '[' '^'? class_item+ ']'
//!   class_item := char | char '-' char | '\' char
//!   backref   := '\' number | '\g{' identifier '}'
//!   escape    := '\' char

use crate::ast::{ClassItem, Expr, Quantifier};
use crate::error::ParseError;
use crate::lexer::{Lexer, Token};

/// Parser for regex patterns
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given input string
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();
        Parser {
            lexer,
            current_token,
        }
    }

    /// Advance to the next token
    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    /// Expect a specific token, error if not found
    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.current_token == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", expected),
                found: self.current_token.to_string(),
            })
        }
    }

    /// Parse the entire input and return the AST
    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_alternation()?;

        // Ensure we've consumed all tokens
        if self.current_token != Token::Eof {
            return Err(ParseError::UnexpectedToken {
                expected: "EOF".to_string(),
                found: self.current_token.to_string(),
            });
        }

        Ok(expr)
    }

    /// Parse alternation (lowest precedence)
    /// alternation := sequence ( '|' sequence )*
    fn parse_alternation(&mut self) -> Result<Expr, ParseError> {
        let mut alternatives = Vec::new();

        // Parse first alternative
        alternatives.push(self.parse_sequence()?);

        // Parse additional alternatives
        while self.current_token == Token::Pipe {
            self.advance(); // consume '|'
            alternatives.push(self.parse_sequence()?);
        }

        if alternatives.len() == 1 {
            Ok(alternatives.into_iter().next().unwrap())
        } else {
            Ok(Expr::Alternation(alternatives))
        }
    }

    /// Parse a sequence (concatenation)
    /// sequence := quantified+
    fn parse_sequence(&mut self) -> Result<Expr, ParseError> {
        let mut expressions = Vec::new();

        // Parse elements until we hit a delimiter
        while !self.is_sequence_end() {
            expressions.push(self.parse_quantified()?);
        }

        if expressions.is_empty() {
            Ok(Expr::Empty)
        } else if expressions.len() == 1 {
            Ok(expressions.into_iter().next().unwrap())
        } else {
            Ok(Expr::Sequence(expressions))
        }
    }

    /// Check if we've reached the end of a sequence
    fn is_sequence_end(&self) -> bool {
        matches!(
            self.current_token,
            Token::Eof | Token::RightParen | Token::RightBracket | Token::RightBrace | Token::Pipe
        )
    }

    /// Parse a quantified expression
    /// quantified := atom quantifier?
    fn parse_quantified(&mut self) -> Result<Expr, ParseError> {
        let atom = self.parse_atom()?;

        // Check for quantifier
        if let Some(quantifier) = self.parse_quantifier()? {
            Ok(Expr::Quantified {
                expr: Box::new(atom),
                quantifier,
            })
        } else {
            Ok(atom)
        }
    }

    /// Parse a quantifier if present
    /// quantifier := '*' | '+?' | '?' | '{' number (',' number?)? '}' '?'?
    fn parse_quantifier(&mut self) -> Result<Option<Quantifier>, ParseError> {
        match &self.current_token {
            Token::Star => {
                self.advance();
                Ok(Some(Quantifier::ZeroOrMore))
            }
            Token::StarLazy => {
                self.advance();
                Ok(Some(Quantifier::ZeroOrMoreLazy))
            }
            Token::Plus => {
                self.advance();
                Ok(Some(Quantifier::OneOrMore))
            }
            Token::PlusLazy => {
                self.advance();
                Ok(Some(Quantifier::OneOrMoreLazy))
            }
            Token::Question => {
                self.advance();
                Ok(Some(Quantifier::Optional))
            }
            Token::LeftBrace => {
                self.advance(); // consume '{'
                let min = self.parse_number()?;

                let quantifier = if self.current_token == Token::Comma {
                    self.advance(); // consume ','
                    if self.current_token == Token::RightBrace {
                        // {n,} - at least n
                        Quantifier::AtLeast(min)
                    } else {
                        // {n,m} - between n and m
                        let max = self.parse_number()?;
                        Quantifier::Between(min, max)
                    }
                } else {
                    // {n} - exactly n
                    Quantifier::Exactly(min)
                };

                self.expect(Token::RightBrace)?;

                // Check for lazy modifier (?) after {n,m} or {n,}
                let quantifier = if self.current_token == Token::Question {
                    self.advance();
                    match quantifier {
                        Quantifier::AtLeast(n) => Quantifier::AtLeastLazy(n),
                        Quantifier::Between(n, m) => Quantifier::BetweenLazy(n, m),
                        _ => quantifier,
                    }
                } else {
                    quantifier
                };

                Ok(Some(quantifier))
            }
            _ => Ok(None),
        }
    }

    /// Parse a number (for quantifiers)
    fn parse_number(&mut self) -> Result<u32, ParseError> {
        match &self.current_token {
            Token::Literal(c) if c.is_ascii_digit() => {
                let mut num = c.to_digit(10).unwrap();
                self.advance();
                // Read additional digits
                while let Token::Literal(c) = &self.current_token {
                    if c.is_ascii_digit() {
                        num = num * 10 + c.to_digit(10).unwrap();
                        self.advance();
                    } else {
                        break;
                    }
                }
                Ok(num)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "number".to_string(),
                found: self.current_token.to_string(),
            }),
        }
    }

    /// Parse an atomic expression
    /// atom := literal | anchor | group | char_class | '.' | backref | escape
    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        match &self.current_token {
            Token::Literal(c) => {
                let expr = Expr::Literal(*c);
                self.advance();
                Ok(expr)
            }
            Token::Dot => {
                self.advance();
                Ok(Expr::Any)
            }
            Token::Caret => {
                self.advance();
                Ok(Expr::StartAnchor)
            }
            Token::Dollar => {
                self.advance();
                Ok(Expr::EndAnchor)
            }
            Token::Escape(c) => {
                // Escaped character - treat as literal for now
                // In full implementation, this could be special (\d, \w, etc.)
                let expr = Expr::Literal(*c);
                self.advance();
                Ok(expr)
            }
            Token::BackrefNumber(n) => {
                let expr = Expr::Backreference(*n);
                self.advance();
                Ok(expr)
            }
            Token::BackrefRelative(n) => {
                let expr = Expr::RelativeBackreference(*n);
                self.advance();
                Ok(expr)
            }
            Token::BackrefName(name) => {
                let expr = Expr::NamedBackreference(name.clone());
                self.advance();
                Ok(expr)
            }
            Token::WordChar => {
                self.advance();
                Ok(Expr::Shorthand('w'))
            }
            Token::NonWordChar => {
                self.advance();
                Ok(Expr::Shorthand('W'))
            }
            Token::Digit => {
                self.advance();
                Ok(Expr::Shorthand('d'))
            }
            Token::NonDigit => {
                self.advance();
                Ok(Expr::Shorthand('D'))
            }
            Token::Whitespace => {
                self.advance();
                Ok(Expr::Shorthand('s'))
            }
            Token::NonWhitespace => {
                self.advance();
                Ok(Expr::Shorthand('S'))
            }
            Token::WordBoundary => {
                self.advance();
                Ok(Expr::WordBoundary)
            }
            Token::NonWordBoundary => {
                self.advance();
                Ok(Expr::NonWordBoundary)
            }
            Token::LeftParen => self.parse_group(),
            Token::NamedGroupStart(name) => {
                // Direct named group without explicit paren handling
                let name = name.clone();
                self.advance();
                let pattern = self.parse_alternation()?;
                self.expect(Token::RightParen)?;
                Ok(Expr::NamedGroup {
                    name,
                    pattern: Box::new(pattern),
                })
            }
            Token::NonCapturing => {
                self.advance();
                let pattern = self.parse_alternation()?;
                self.expect(Token::RightParen)?;
                Ok(Expr::NonCapturingGroup(Box::new(pattern)))
            }
            Token::Lookahead => {
                self.advance();
                let pattern = self.parse_alternation()?;
                self.expect(Token::RightParen)?;
                Ok(Expr::Lookahead(Box::new(pattern)))
            }
            Token::NegativeLookahead => {
                self.advance();
                let pattern = self.parse_alternation()?;
                self.expect(Token::RightParen)?;
                Ok(Expr::NegativeLookahead(Box::new(pattern)))
            }
            Token::Lookbehind => {
                self.advance();
                let pattern = self.parse_alternation()?;
                self.expect(Token::RightParen)?;
                Ok(Expr::Lookbehind(Box::new(pattern)))
            }
            Token::NegativeLookbehind => {
                self.advance();
                let pattern = self.parse_alternation()?;
                self.expect(Token::RightParen)?;
                Ok(Expr::NegativeLookbehind(Box::new(pattern)))
            }
            Token::Atomic => {
                self.advance();
                let pattern = self.parse_alternation()?;
                self.expect(Token::RightParen)?;
                Ok(Expr::AtomicGroup(Box::new(pattern)))
            }
            Token::Conditional => {
                self.advance();
                let pattern = self.parse_alternation()?;
                self.expect(Token::RightParen)?;
                Ok(Expr::ConditionalGroup(Box::new(pattern)))
            }
            Token::ModeFlags(flags) => {
                let flags_owned = flags.to_string();
                self.advance();
                let pattern = self.parse_alternation()?;
                self.expect(Token::RightParen)?;
                Ok(Expr::ModeFlagsGroup { flags: flags_owned, pattern: Box::new(pattern) })
            }
            Token::LeftBracket => self.parse_char_class(),
            Token::Eof => Err(ParseError::UnexpectedEof),
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: self.current_token.to_string(),
            }),
        }
    }

    /// Parse a group
    /// group := '(' group_inner ')'
    fn parse_group(&mut self) -> Result<Expr, ParseError> {
        self.expect(Token::LeftParen)?;

        let expr = match &self.current_token {
            Token::NamedGroupStart(name) => {
                let name = name.clone();
                self.advance(); // consume the name and colon
                let pattern = self.parse_alternation()?;
                Expr::NamedGroup {
                    name,
                    pattern: Box::new(pattern),
                }
            }
            Token::NonCapturing => {
                self.advance(); // consume '?:'
                let pattern = self.parse_alternation()?;
                Expr::NonCapturingGroup(Box::new(pattern))
            }
            _ => {
                // Regular capturing group
                let pattern = self.parse_alternation()?;
                Expr::Group(Box::new(pattern))
            }
        };

        self.expect(Token::RightParen)?;
        Ok(expr)
    }

    /// Parse a character class
    /// char_class := '[' '^'? class_item+ ']'
    fn parse_char_class(&mut self) -> Result<Expr, ParseError> {
        self.expect(Token::LeftBracket)?;

        let negated = if self.current_token == Token::Caret {
            self.advance();
            true
        } else {
            false
        };

        let mut items = Vec::new();

        // Parse class items until we hit ']'
        while self.current_token != Token::RightBracket && self.current_token != Token::Eof {
            items.push(self.parse_class_item()?);
        }

        if items.is_empty() {
            return Err(ParseError::UnexpectedToken {
                expected: "character class item".to_string(),
                found: self.current_token.to_string(),
            });
        }

        self.expect(Token::RightBracket)?;

        Ok(Expr::CharacterClass(crate::ast::CharacterClass {
            negated,
            items,
        }))
    }

    /// Parse an item in a character class
    /// class_item := char | char '-' char | '\' char
    fn parse_class_item(&mut self) -> Result<ClassItem, ParseError> {
        match &self.current_token {
            Token::Literal(c) => {
                let start = *c;
                self.advance();

                // Check for range (e.g., a-z)
                if matches!(self.current_token, Token::Literal('-')) {
                    self.advance(); // consume '-'
                    if let Token::Literal(end) = &self.current_token {
                        let end = *end;
                        self.advance();
                        Ok(ClassItem::Range(start, end))
                    } else {
                        // Not a range, treat '-' as literal
                        Ok(ClassItem::Char(start))
                    }
                } else {
                    Ok(ClassItem::Char(start))
                }
            }
            Token::Escape(c) => {
                // Escaped character in class (could be \d, \w, etc.)
                let c = *c;
                self.advance();
                Ok(ClassItem::Shorthand(c))
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "character or escape".to_string(),
                found: self.current_token.to_string(),
            }),
        }
    }
}

/// Parse a regex pattern string into an AST
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let mut parser = Parser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Quantifier;

    #[test]
    fn test_parse_literal() {
        let expr = parse("abc").unwrap();
        assert_eq!(expr.to_regex_string(), "abc");
    }

    #[test]
    fn test_parse_named_group() {
        let expr = parse("(name:abc)").unwrap();
        assert_eq!(expr.to_regex_string(), "(?<name>abc)");
    }

    #[test]
    fn test_parse_empty() {
        let expr = parse("").unwrap();
        assert_eq!(expr.to_regex_string(), "");
    }

    #[test]
    fn test_parse_nested_group() {
        let expr = parse("(outer:(inner:abc))").unwrap();
        assert_eq!(expr.to_regex_string(), "(?<outer>(?<inner>abc))");
    }

    #[test]
    fn test_parse_error_unclosed_group() {
        let result = parse("(name:abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_simple_parens() {
        // (abc) - capturing group
        let expr = parse("(abc)").unwrap();
        assert_eq!(expr.to_regex_string(), "(abc)");
    }

    #[test]
    fn test_parse_quantifier_star() {
        let expr = parse("a*").unwrap();
        assert_eq!(expr.to_regex_string(), "a*");
        assert!(matches!(
            expr,
            Expr::Quantified {
                quantifier: Quantifier::ZeroOrMore,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_quantifier_plus() {
        let expr = parse("a+").unwrap();
        assert_eq!(expr.to_regex_string(), "a+");
    }

    #[test]
    fn test_parse_quantifier_optional() {
        let expr = parse("a?").unwrap();
        assert_eq!(expr.to_regex_string(), "a?");
    }

    #[test]
    fn test_parse_quantifier_exact() {
        let expr = parse("a{3}").unwrap();
        assert_eq!(expr.to_regex_string(), "a{3}");
    }

    #[test]
    fn test_parse_quantifier_between() {
        let expr = parse("a{2,5}").unwrap();
        assert_eq!(expr.to_regex_string(), "a{2,5}");
    }

    #[test]
    fn test_parse_quantifier_at_least() {
        let expr = parse("a{3,}").unwrap();
        assert_eq!(expr.to_regex_string(), "a{3,}");
    }

    #[test]
    fn test_parse_alternation() {
        let expr = parse("a|b|c").unwrap();
        assert_eq!(expr.to_regex_string(), "a|b|c");
        assert!(matches!(expr, Expr::Alternation(_)));
    }

    #[test]
    fn test_parse_anchors() {
        let expr = parse("^start$").unwrap();
        assert_eq!(expr.to_regex_string(), "^start$");
    }

    #[test]
    fn test_parse_dot() {
        let expr = parse("a.b").unwrap();
        assert_eq!(expr.to_regex_string(), "a.b");
    }

    #[test]
    fn test_parse_non_capturing_group() {
        let expr = parse("(?:abc)").unwrap();
        assert_eq!(expr.to_regex_string(), "(?:abc)");
    }

    #[test]
    fn test_parse_backreference_number() {
        let expr = parse(r"\1").unwrap();
        assert_eq!(expr.to_regex_string(), "\\1");
    }

    #[test]
    fn test_parse_backreference_relative() {
        let expr = parse(r"\g{-1}").unwrap();
        assert_eq!(expr.to_regex_string(), "\\g{-1}");
    }

    #[test]
    fn test_parse_backreference_name() {
        let expr = parse(r"\g{name}").unwrap();
        assert_eq!(expr.to_regex_string(), "\\g{name}");
    }

    #[test]
    fn test_parse_character_class() {
        let expr = parse("[abc]").unwrap();
        assert_eq!(expr.to_regex_string(), "[abc]");
    }

    #[test]
    fn test_parse_character_class_negated() {
        let expr = parse("[^abc]").unwrap();
        assert_eq!(expr.to_regex_string(), "[^abc]");
    }

    #[test]
    fn test_parse_character_class_range() {
        let expr = parse("[a-z]").unwrap();
        assert_eq!(expr.to_regex_string(), "[a-z]");
    }

    #[test]
    fn test_parse_complex_pattern() {
        // Test a more complex pattern with multiple features
        let expr = parse("^(name:[a-z]+)@(domain:[a-z]+)$").unwrap();
        assert_eq!(
            expr.to_regex_string(),
            "^(?<name>[a-z]+)@(?<domain>[a-z]+)$"
        );
    }
}
