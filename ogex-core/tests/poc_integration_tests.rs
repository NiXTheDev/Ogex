//! Integration tests for the proof-of-concept
//!
//! These tests validate that the PoC components work together correctly.

use ogex_core::{compile, parse, transpile_debug, Lexer};

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
fn test_poc_mixed_content() {
    let result = compile("prefix(name:middle)suffix").unwrap();
    assert_eq!(result, "prefix(?<name>middle)suffix");
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
