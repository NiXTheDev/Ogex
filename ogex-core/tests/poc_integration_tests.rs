//! Integration tests for the proof-of-concept
//!
//! These tests validate that the PoC components work together correctly.

use ogex_core::{compile, parse, transpile_debug, Lexer, Regex, Replacement};

#[test]
fn test_poc_full_pipeline() {
    // Test the complete pipeline: input -> tokens -> AST -> output
    let input = "(name:abc)";

    // Step 1: Lexing
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    assert!(!tokens.is_empty());

    // Step 2: Parsing
    let _ast = parse(input).unwrap();

    // Step 3: Transpilation
    let output = compile(input).unwrap();

    // Verify
    assert_eq!(output, "(?<name>abc)");
}

#[test]
fn test_poc_various_named_groups() {
    let test_cases = vec![
        ("(x:a)", "(?<x>a)"),
        ("(username:john)", "(?<username>john)"),
        ("(group_name:test)", "(?<group_name>test)"),
        ("(n1:123)", "(?<n1>123)"),
    ];

    for (input, expected) in test_cases {
        let result = compile(input).unwrap();
        assert_eq!(result, expected, "Failed for input: {}", input);
    }
}

#[test]
fn test_poc_nested_groups() {
    let test_cases = vec![
        ("(outer:(inner:a))", "(?<outer>(?<inner>a))"),
        ("(a:(b:(c:d)))", "(?<a>(?<b>(?<c>d)))"),
        (
            "(first:(second:hello)world)",
            "(?<first>(?<second>hello)world)",
        ),
    ];

    for (input, expected) in test_cases {
        let result = compile(input).unwrap();
        assert_eq!(result, expected, "Failed for input: {}", input);
    }
}

#[test]
fn test_poc_literal_sequences() {
    assert_eq!(compile("abc").unwrap(), "abc");
    assert_eq!(compile("hello world").unwrap(), "hello world");
    assert_eq!(compile("12345").unwrap(), "12345");
    assert_eq!(compile("hello_world").unwrap(), "hello_world");
}

#[test]
fn test_named_group_replacement() {
    // Test basic numbered group replacement
    let regex = Regex::new(r"(a)(b)").unwrap();
    let m = regex.find("ab").unwrap();

    // Replace with groups swapped
    let repl = Replacement::parse(r"\g{2}\g{1}").unwrap();

    // Groups must be in order by index (group 1 first, group 2 second)
    let mut group_pairs = vec![(0usize, 0usize); m.groups.len()];
    for (&idx, &(s, e)) in &m.groups {
        if idx > 0 && (idx as usize) <= group_pairs.len() {
            group_pairs[(idx - 1) as usize] = (s, e);
        }
    }
    let result = repl.apply("ab", m.start, m.end, &group_pairs);
    // Result should be "ba" (swapped)
    assert_eq!(result, "ba");
}

#[test]
fn test_poc_error_handling() {
    // Unclosed group
    assert!(compile("(name:abc").is_err());

    // These should work (empty is valid)
    assert!(compile("").is_ok());
}

#[test]
fn test_poc_debug_output() {
    let result = transpile_debug("(name:test)").unwrap();

    assert_eq!(result.input, "(name:test)");
    assert_eq!(result.output, "(?<name>test)");
    // AST should contain the pattern
    assert!(result.ast.contains("NamedGroup"));
    assert!(result.ast.contains("name"));
}

#[test]
fn test_poc_complex_real_world_patterns() {
    // Simulating real-world use cases (simplified for PoC)

    // Simple patterns with named groups
    let pattern1 = "(user:john)(domain:example)";
    let result = compile(pattern1).unwrap();
    assert!(result.contains("(?<user>"));
    assert!(result.contains("(?<domain>"));

    // URL-like pattern (simplified, no special chars)
    let pattern2 = "(protocol:https)(host:example)";
    let result = compile(pattern2).unwrap();
    assert!(result.contains("(?<protocol>"));
    assert!(result.contains("(?<host>"));
}

#[test]
fn test_poc_special_characters_in_literals() {
    // Note: Using simple patterns since full parser isn't implemented yet
    let test_cases = vec![
        ("abc", "abc"),                 // Simple literals
        ("hello world", "hello world"), // Spaces preserved
        ("test123", "test123"),         // Alphanumeric
    ];

    for (input, expected) in test_cases {
        let result = compile(input).unwrap();
        assert_eq!(result, expected, "Failed for input: {}", input);
    }
}

#[test]
fn test_poc_empty_groups() {
    // Empty named group
    let result = compile("(name:)").unwrap();
    assert_eq!(result, "(?<name>)");

    // Empty simple parens (capturing group)
    let result = compile("()").unwrap();
    assert_eq!(result, "()");
}

#[test]
fn test_poc_unicode_support() {
    // Unicode in literals should be preserved
    let result = compile("(name:こんにちは)").unwrap();
    assert_eq!(result, "(?<name>こんにちは)");

    let result = compile("(emoji:🎉)").unwrap();
    assert_eq!(result, "(?<emoji>🎉)");
}

#[test]
fn test_poc_idempotency() {
    // The transpiler should handle the same pattern consistently
    let input = "(name:value)";
    let result1 = compile(input).unwrap();
    let result2 = compile(input).unwrap();
    assert_eq!(result1, result2);
}

#[test]
fn test_poc_performance_smoke() {
    // Smoke test to ensure we're not pathologically slow
    use std::time::Instant;

    let input = "(a:(b:(c:(d:(e:deep)))))";
    let start = Instant::now();

    for _ in 0..100 {
        let _ = compile(input).unwrap();
    }

    let elapsed = start.elapsed();
    // Should complete 100 iterations in under 1 second
    assert!(
        elapsed.as_secs() < 1,
        "Performance test took too long: {:?}",
        elapsed
    );
}

// =============================================================================
// Integration tests for \g{-n} relative backreferences and \G entire match
// =============================================================================

#[test]
fn test_relative_backref_basic() {
    // Pattern: (a)(b)\g{-1} should match "abb" (last numbered group is "b")
    let regex = Regex::new(r"(a)(b)\g{-1}").unwrap();
    assert!(regex.is_match("abb"));
    assert!(!regex.is_match("aba"));
    assert!(!regex.is_match("abc"));
}

#[test]
fn test_relative_backref_multiple_groups() {
    // Pattern: (a)(b)(c)\g{-1} should match "abcc" (last numbered group is "c")
    let regex = Regex::new(r"(a)(b)(c)\g{-1}").unwrap();
    assert!(regex.is_match("abcc"));
    assert!(!regex.is_match("abcb"));
}

#[test]
fn test_relative_backref_with_named_groups() {
    // Named groups are excluded from relative indexing
    // Pattern: (a)(name:x)(b)\g{-2}
    // Groups: 1=(a) numbered, 2=(name:x) named, 3=(b) numbered
    // Numbered groups: [1, 3]
    // \g{-2} = second-to-last numbered = group 1 = "a"
    // If named were included, \g{-2} would be group 2 = "x"
    let regex = Regex::new(r"(a)(name:x)(b)\g{-2}").unwrap();
    assert!(regex.is_match("axba")); // a, x, b, a (matches because \g{-2}="a")
    assert!(!regex.is_match("axbx")); // would match if named groups were included (\g{-2}="x")
}

#[test]
fn test_relative_backref_second_to_last() {
    // \g{-2} = second-to-last numbered group
    // Pattern: (a)(b)(c)\g{-2}
    // Groups: 1=a, 2=b, 3=c (all numbered)
    // Numbered groups indices: [1, 2, 3]
    // \g{-2} = second-to-last = group 2 = "b"
    let regex = Regex::new(r"(a)(b)(c)\g{-2}").unwrap();
    assert!(regex.is_match("abcb")); // a, b, c, b
    assert!(!regex.is_match("abca"));
}

#[test]
fn test_relative_backref_find() {
    // Use a numbered group, not named
    let regex = Regex::new(r"(\d\d\d+)\g{-1}").unwrap();
    // Should match two consecutive same 3+ digit numbers
    let m = regex.find("test 123123 end");
    assert!(m.is_some());
    let m = m.unwrap();
    assert_eq!(m.as_str("test 123123 end"), "123123");
}

#[test]
fn test_relative_backref_in_sequence() {
    // Multiple relative backrefs
    // Pattern: (a)(b)\g{-1}\g{-2} should match "abba"
    let regex = Regex::new(r"(a)(b)\g{-1}\g{-2}").unwrap();
    assert!(regex.is_match("abba"));
    assert!(!regex.is_match("abab"));
}

#[test]
fn test_entire_match_replacement_g() {
    // \G in replacement = entire match
    let repl = Replacement::parse(r"[\G]").unwrap();
    let result = repl.apply("hello world", 0, 5, &[]);
    assert_eq!(result, "[hello]");
}

#[test]
fn test_entire_match_replacement_g_with_groups() {
    // Simple test - \G in replacement
    let regex = Regex::new(r"\w+").unwrap();
    let m = regex.find("hello world").unwrap();

    // Replace with entire match wrapped
    let repl = Replacement::parse(r"<\G>").unwrap();
    let group_pairs: Vec<_> = m.groups.values().map(|(s, e)| (*s, *e)).collect();
    let result = repl.apply("hello world", m.start, m.end, &group_pairs);
    assert_eq!(result, "<hello>");
}

#[test]
fn test_numbered_group_replacement() {
    // Test using numbered groups directly
    let regex = Regex::new(r"(a)(b)(c)").unwrap();
    let m = regex.find("abc").unwrap();

    let repl = Replacement::parse(r"\g{2}, \g{1}, \g{3}").unwrap();

    // Groups in order by index
    let mut group_pairs = vec![(0usize, 0usize); m.groups.len()];
    for (&idx, &(s, e)) in &m.groups {
        if idx > 0 && idx <= m.groups.len() as u32 {
            group_pairs[(idx - 1) as usize] = (s, e);
        }
    }
    let result = repl.apply("abc", m.start, m.end, &group_pairs);
    // Group 1 = "a", Group 2 = "b", Group 3 = "c"
    assert_eq!(result, "b, a, c");
}

#[test]
fn test_escape_g_in_pattern() {
    // \G in a pattern is a literal 'G' (escaped)
    let regex = Regex::new(r"\G").unwrap();
    assert!(regex.is_match("G"));
    assert!(!regex.is_match("g")); // case-sensitive
    assert!(!regex.is_match("abc"));
}

#[test]
fn test_relative_backref_transpilation() {
    // Relative backrefs should be preserved in transpilation
    let result = compile(r"(a)\g{-1}").unwrap();
    assert!(result.contains(r"\g{-1}"));
}

#[test]
fn test_cli_style_pattern_matching() {
    // Test patterns that would be used via CLI
    let patterns_and_inputs = vec![
        (r"(a)(b)\g{-1}", "abb", true),
        (r"(a)(b)\g{-1}", "aba", false),
        (r"(x)(y)\g{-1}", "xyy", true), // numbered groups, not named
        (r"(\d\d\d)\g{-1}", "123123", true), // numbered group, not named
    ];

    for (pattern, input, should_match) in patterns_and_inputs {
        let regex = Regex::new(pattern).unwrap();
        assert_eq!(
            regex.is_match(input),
            should_match,
            "Pattern {} with input {} expected match={}",
            pattern,
            input,
            should_match
        );
    }
}
