//! Lexer for tokenizing regex patterns
//!
//! This module provides a tokenizer that converts regex pattern strings
//! into a stream of tokens for parsing.

use std::fmt;

/// A token in a regex pattern
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Left parenthesis `(`
    LeftParen,
    /// Right parenthesis `)`
    RightParen,
    /// Left bracket `[` (start of character class)
    LeftBracket,
    /// Right bracket `]` (end of character class)
    RightBracket,
    /// Left brace `{` (start of quantifier)
    LeftBrace,
    /// Right brace `}` (end of quantifier)
    RightBrace,
    /// Colon `:`
    Colon,
    /// Comma `,` (used in quantifiers like {n,m})
    Comma,
    /// Pipe `|` (alternation)
    Pipe,
    /// Caret `^` (start anchor or negation in character class)
    Caret,
    /// Dollar `$` (end anchor)
    Dollar,
    /// Dot `.` (any character)
    Dot,
    /// Star `*` (zero or more)
    Star,
    /// Star lazy `*?` (zero or more lazy)
    StarLazy,
    /// Plus `+` (one or more)
    Plus,
    /// Plus lazy `+?` (one or more lazy)
    PlusLazy,
    /// Question `?` (optional)
    Question,
    /// Non-capturing group marker `(?:`
    NonCapturing,
    /// A named group identifier (the name part in `(name:...)`)
    NamedGroupStart(String),
    /// An escaped character (e.g., \n, \t, \\, etc.)
    Escape(char),
    /// A backreference by number (e.g., \1, \2)
    BackrefNumber(u32),
    /// A relative backreference by negative index (e.g., \g{-1}, \g{-2})
    /// References numbered groups only, from the end: \g{-1} = last numbered group
    BackrefRelative(i32),
    /// A backreference by name (e.g., \g{name})
    BackrefName(String),
    /// Word character shorthand (\w)
    WordChar,
    /// Non-word character shorthand (\W)
    NonWordChar,
    /// Digit shorthand (\d)
    Digit,
    /// Non-digit shorthand (\D)
    NonDigit,
    /// Whitespace shorthand (\s)
    Whitespace,
    /// Non-whitespace shorthand (\S)
    NonWhitespace,
    /// Word boundary assertion (\b)
    WordBoundary,
    /// Non-word boundary assertion (\B)
    NonWordBoundary,
    /// A literal character
    Literal(char),
    /// End of input
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::LeftParen => write!(f, "`(`"),
            Token::RightParen => write!(f, "`)`"),
            Token::LeftBracket => write!(f, "`[`"),
            Token::RightBracket => write!(f, "`]`"),
            Token::LeftBrace => write!(f, "`{{`"),
            Token::RightBrace => write!(f, "`}}`"),
            Token::Colon => write!(f, "`:`"),
            Token::Comma => write!(f, "`,`"),
            Token::Pipe => write!(f, "`|`"),
            Token::Caret => write!(f, "`^`"),
            Token::Dollar => write!(f, "`$`"),
            Token::Dot => write!(f, "`.`"),
            Token::Star => write!(f, "`*`"),
            Token::StarLazy => write!(f, "`*?`"),
            Token::Plus => write!(f, "`+`"),
            Token::PlusLazy => write!(f, "`+?`"),
            Token::Question => write!(f, "`?`"),
            Token::NonCapturing => write!(f, "`?:`"),
            Token::NamedGroupStart(name) => write!(f, "named group `{}`", name),
            Token::Escape(c) => write!(f, "escape `\\{}`", c),
            Token::BackrefNumber(n) => write!(f, "backref `\\{}`", n),
            Token::BackrefRelative(n) => write!(f, "relative backref `\\g{{{}}}`", n),
            Token::BackrefName(name) => write!(f, "backref `\\g{{{}}}`", name),
            Token::WordChar => write!(f, "shorthand `\\w`"),
            Token::NonWordChar => write!(f, "shorthand `\\W`"),
            Token::Digit => write!(f, "shorthand `\\d`"),
            Token::NonDigit => write!(f, "shorthand `\\D`"),
            Token::Whitespace => write!(f, "shorthand `\\s`"),
            Token::NonWhitespace => write!(f, "shorthand `\\S`"),
            Token::WordBoundary => write!(f, "boundary `\\b`"),
            Token::NonWordBoundary => write!(f, "boundary `\\B`"),
            Token::Literal(c) => write!(f, "literal `{}`", c),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

/// Lexer for tokenizing regex patterns
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    current_char: Option<char>,
    /// Whether we're currently inside a character class
    in_char_class: bool,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given input string
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            current_char: None,
            in_char_class: false,
        };
        lexer.advance();
        lexer
    }

    /// Advance to the next character
    fn advance(&mut self) {
        self.current_char = self.input.chars().nth(self.position);
        self.position += 1;
    }

    /// Peek at the next character without consuming it
    #[allow(dead_code)]
    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    /// Check if a character is valid for an identifier (group name)
    fn is_identifier_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    /// Read an identifier (group name)
    fn read_identifier(&mut self) -> String {
        let start = self.position - 1; // We already consumed the first char
        while let Some(c) = self.current_char {
            if Self::is_identifier_char(c) {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.position - 1].to_string()
    }

    /// Read a number (for backreferences or quantifiers)
    #[allow(dead_code)]
    fn read_number(&mut self) -> u32 {
        let start = self.position - 1;
        while let Some(c) = self.current_char {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.position - 1].parse().unwrap_or(0)
    }

    /// Read an escape sequence (assumes backslash was already consumed)
    fn read_escape(&mut self) -> Token {
        match self.current_char {
            Some(c) => {
                self.advance();
                // Check for character class shorthands first
                match c {
                    'w' => Token::WordChar,
                    'W' => Token::NonWordChar,
                    'd' => Token::Digit,
                    'D' => Token::NonDigit,
                    's' => Token::Whitespace,
                    'S' => Token::NonWhitespace,
                    'b' => Token::WordBoundary,
                    'B' => Token::NonWordBoundary,
                    _ if c.is_ascii_digit() => {
                        // It's a backreference number
                        let mut num = c.to_digit(10).unwrap();
                        while let Some(c) = self.current_char {
                            if c.is_ascii_digit() {
                                num = num * 10 + c.to_digit(10).unwrap();
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        Token::BackrefNumber(num)
                    }
                    _ => Token::Escape(c),
                }
            }
            None => Token::Escape('\0'), // Should be an error, but for now
        }
    }

    /// Read a backreference with \g{name} syntax
    fn read_g_backref(&mut self) -> Token {
        // Assumes we've already consumed '\' and 'g' and '{'
        // current_char is at the first character of the name
        let start = self.position - 1; // position - 1 is where current_char is
        while let Some(c) = self.current_char {
            if c != '}' {
                self.advance();
            } else {
                break;
            }
        }
        // Now position points past the last character of name, and current_char is '}'
        let content = self.input[start..self.position - 1].to_string();
        if self.current_char == Some('}') {
            self.advance(); // consume '}'
        }

        // Check if this is a relative backreference (negative index)
        // \g{-n} where n is a positive integer
        if let Some(stripped) = content.strip_prefix('-')
            && let Ok(n) = stripped.parse::<i32>()
            && n > 0
        {
            return Token::BackrefRelative(-n); // Store as negative: -1, -2, etc.
        }

        // Default: treat as named backreference (including \g{1} for positive numbers)
        Token::BackrefName(content)
    }

    /// Get the next token from the input
    pub fn next_token(&mut self) -> Token {
        match self.current_char {
            None => Token::Eof,
            Some('\\') => {
                self.advance(); // consume backslash
                match self.current_char {
                    Some('g') => {
                        self.advance(); // consume 'g'
                        if self.current_char == Some('{') {
                            self.advance(); // consume '{'
                            self.read_g_backref()
                        } else {
                            // Read 'g' as escape, but check if followed by digits
                            Token::Escape('g')
                        }
                    }
                    Some(_c) => self.read_escape(),
                    None => Token::Escape('\0'),
                }
            }
            Some('(') => {
                // Look ahead to check if this is a named group or non-capturing
                let start_pos = self.position;
                self.advance(); // consume '('

                if self.current_char == Some('?') {
                    self.advance(); // consume '?'
                    if self.current_char == Some(':') {
                        self.advance(); // consume ':'
                        return Token::NonCapturing;
                    } else {
                        // Reset and return LeftParen
                        self.position = start_pos;
                        self.current_char = Some('(');
                        self.advance();
                        return Token::LeftParen;
                    }
                }

                if let Some(c) = self.current_char
                    && (c.is_alphabetic() || c == '_')
                {
                    let name = self.read_identifier();
                    // After the name, we expect a colon for named group
                    if self.current_char == Some(':') {
                        self.advance(); // consume the colon
                        return Token::NamedGroupStart(name);
                    }
                }

                // Not a special group, reset and return LeftParen
                self.position = start_pos;
                self.current_char = Some('(');
                self.advance();
                Token::LeftParen
            }
            Some(')') => {
                self.advance();
                self.in_char_class = false;
                Token::RightParen
            }
            Some('[') => {
                self.advance();
                self.in_char_class = true;
                Token::LeftBracket
            }
            Some(']') => {
                self.advance();
                self.in_char_class = false;
                Token::RightBracket
            }
            Some('{') => {
                self.advance();
                Token::LeftBrace
            }
            Some('}') => {
                self.advance();
                Token::RightBrace
            }
            Some(':') => {
                self.advance();
                Token::Colon
            }
            Some(',') => {
                self.advance();
                Token::Comma
            }
            Some('|') => {
                self.advance();
                Token::Pipe
            }
            Some('^') => {
                self.advance();
                Token::Caret
            }
            Some('$') => {
                self.advance();
                Token::Dollar
            }
            Some('.') => {
                self.advance();
                Token::Dot
            }
            Some('*') => {
                self.advance();
                if self.current_char == Some('?') {
                    self.advance();
                    Token::StarLazy
                } else {
                    Token::Star
                }
            }
            Some('+') => {
                self.advance();
                if self.current_char == Some('?') {
                    self.advance();
                    Token::PlusLazy
                } else {
                    Token::Plus
                }
            }
            Some('?') => {
                self.advance();
                Token::Question
            }
            Some(c) => {
                self.advance();
                Token::Literal(c)
            }
        }
    }

    /// Tokenize the entire input and return a vector of tokens
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_named_group() {
        let mut lexer = Lexer::new("(name:abc)");
        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![
                Token::NamedGroupStart("name".to_string()),
                Token::Literal('a'),
                Token::Literal('b'),
                Token::Literal('c'),
                Token::RightParen,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_literal_sequence() {
        let mut lexer = Lexer::new("abc");
        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![
                Token::Literal('a'),
                Token::Literal('b'),
                Token::Literal('c'),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize();

        assert_eq!(tokens, vec![Token::Eof]);
    }

    #[test]
    fn test_parentheses_only() {
        let mut lexer = Lexer::new("()");
        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![Token::LeftParen, Token::RightParen, Token::Eof,]
        );
    }

    #[test]
    fn test_named_group_with_underscore() {
        let mut lexer = Lexer::new("(my_name:test)");
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0], Token::NamedGroupStart("my_name".to_string()));
    }

    #[test]
    fn test_quantifiers() {
        let mut lexer = Lexer::new("a*b+c?");
        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![
                Token::Literal('a'),
                Token::Star,
                Token::Literal('b'),
                Token::Plus,
                Token::Literal('c'),
                Token::Question,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_alternation() {
        let mut lexer = Lexer::new("a|b|c");
        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![
                Token::Literal('a'),
                Token::Pipe,
                Token::Literal('b'),
                Token::Pipe,
                Token::Literal('c'),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_anchors() {
        let mut lexer = Lexer::new("^start$");
        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![
                Token::Caret,
                Token::Literal('s'),
                Token::Literal('t'),
                Token::Literal('a'),
                Token::Literal('r'),
                Token::Literal('t'),
                Token::Dollar,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_character_class() {
        let mut lexer = Lexer::new("[abc]");
        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![
                Token::LeftBracket,
                Token::Literal('a'),
                Token::Literal('b'),
                Token::Literal('c'),
                Token::RightBracket,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_escape_sequences() {
        let mut lexer = Lexer::new(r"\n\t\\");
        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![
                Token::Escape('n'),
                Token::Escape('t'),
                Token::Escape('\\'),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_backreference_number() {
        let mut lexer = Lexer::new(r"\1\2\12");
        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![
                Token::BackrefNumber(1),
                Token::BackrefNumber(2),
                Token::BackrefNumber(12),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_backreference_name() {
        let mut lexer = Lexer::new(r"\g{name}");
        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![Token::BackrefName("name".to_string()), Token::Eof,]
        );
    }

    #[test]
    fn test_non_capturing_group() {
        let mut lexer = Lexer::new("(?:abc)");
        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![
                Token::NonCapturing,
                Token::Literal('a'),
                Token::Literal('b'),
                Token::Literal('c'),
                Token::RightParen,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_dot() {
        let mut lexer = Lexer::new("a.b");
        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![
                Token::Literal('a'),
                Token::Dot,
                Token::Literal('b'),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_quantifier_braces() {
        let mut lexer = Lexer::new("a{3,5}");
        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![
                Token::Literal('a'),
                Token::LeftBrace,
                Token::Literal('3'),
                Token::Comma,
                Token::Literal('5'),
                Token::RightBrace,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_backref_relative() {
        // \g{-1} - relative backreference to last numbered group
        let mut lexer = Lexer::new(r"\g{-1}");
        let tokens = lexer.tokenize();

        assert_eq!(tokens, vec![Token::BackrefRelative(-1), Token::Eof,]);
    }

    #[test]
    fn test_backref_relative_large() {
        // \g{-27} - relative backreference
        let mut lexer = Lexer::new(r"\g{-27}");
        let tokens = lexer.tokenize();

        assert_eq!(tokens, vec![Token::BackrefRelative(-27), Token::Eof,]);
    }

    #[test]
    fn test_backref_relative_in_pattern() {
        // Pattern with relative backreference
        let mut lexer = Lexer::new(r"(a)(b)\g{-1}");
        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![
                Token::LeftParen,
                Token::Literal('a'),
                Token::RightParen,
                Token::LeftParen,
                Token::Literal('b'),
                Token::RightParen,
                Token::BackrefRelative(-1),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_escape_uppercase_g() {
        // \G in patterns is a literal G (escaped character)
        let mut lexer = Lexer::new(r"\G");
        let tokens = lexer.tokenize();

        assert_eq!(tokens, vec![Token::Escape('G'), Token::Eof,]);
    }

    #[test]
    fn test_escape_g_no_brace() {
        // \g without { is just an escaped 'g'
        let mut lexer = Lexer::new(r"\g");
        let tokens = lexer.tokenize();

        assert_eq!(tokens, vec![Token::Escape('g'), Token::Eof,]);
    }

    #[test]
    fn test_backref_name_with_number() {
        // \g{1} is treated as a named backreference with name "1"
        let mut lexer = Lexer::new(r"\g{1}");
        let tokens = lexer.tokenize();

        assert_eq!(
            tokens,
            vec![Token::BackrefName("1".to_string()), Token::Eof,]
        );
    }
}
