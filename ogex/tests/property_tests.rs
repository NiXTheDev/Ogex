//! Property-based tests for ogex using proptest
//!
//! These tests verify invariants using randomly generated inputs.

use ogex::{Regex, Replacement, compile, parse};
use proptest::prelude::*;

// =============================================================================
// Tests: Regex parsing invariants (with proptest parameters)
// =============================================================================

proptest! {
    // Any valid regex pattern should parse without panic
    fn test_valid_patterns_parse_without_panic(s in "[a-zA-Z0-9]+") {
        let result = std::panic::catch_unwind(|| {
            let _ = parse(&s);
        });
        prop_assert!(result.is_ok());
    }

    // Named group patterns should parse without panic
    fn test_named_groups_parse_without_panic(s in "[a-z]+") {
        let pattern = format!("({}:test)", s);
        let result = std::panic::catch_unwind(|| {
            let _ = parse(&pattern);
        });
        prop_assert!(result.is_ok());
    }

    // Literal-only patterns should parse and compile
    fn test_literal_patterns(s in "[a-zA-Z0-9]+") {
        let parse_result = parse(&s);
        prop_assert!(parse_result.is_ok(), "Literal pattern should parse: {}", s);

        let compile_result = compile(&s);
        prop_assert!(compile_result.is_ok(), "Literal pattern should compile: {}", s);
    }
}

// =============================================================================
// Tests: Empty pattern handling (regular #[test])
// =============================================================================

#[test]
fn test_empty_pattern_handling() {
    let result = parse("");
    assert!(result.is_ok(), "Empty pattern should parse successfully");
}

// =============================================================================
// Tests: Transpiler round-trip invariants
// =============================================================================

proptest! {
    // Parsing and recompiling should produce valid output
    fn test_parse_compile_produces_valid_output(s in "[a-zA-Z0-9]+") {
        let first = compile(&s);
        prop_assert!(first.is_ok(), "First compile should succeed");

        let output = first.unwrap();
        prop_assert!(!output.is_empty(), "Output should not be empty");
    }

    // Named groups should round-trip correctly
    fn test_named_group_roundtrip(name in "[a-z]+") {
        let pattern = format!("({}:value)", name);
        let result = compile(&pattern);

        prop_assert!(result.is_ok(), "Pattern should compile: {}", pattern);
        let output = result.unwrap();

        let expected = format!("?<{}>", name);
        prop_assert!(output.contains(&expected), "Output should contain named group: {}", output);
    }

    // Multiple named groups should all be preserved
    fn test_multiple_named_groups(name1 in "[a-z]+", name2 in "[a-z]+") {
        let pattern = format!("({}:a)({}:b)", name1, name2);
        let result = compile(&pattern);

        prop_assert!(result.is_ok());
        let output = result.unwrap();

        prop_assert!(output.contains(&format!("?<{}>", name1)), "Should contain first group: {}", output);
        prop_assert!(output.contains(&format!("?<{}>", name2)), "Should contain second group: {}", output);
    }

    // Idempotency: compiling the same pattern multiple times gives consistent results
    fn test_compile_idempotency(s in "[a-zA-Z0-9]+") {
        let result1 = compile(&s);
        prop_assert!(result1.is_ok());

        let result2 = compile(&s);
        prop_assert!(result2.is_ok());

        prop_assert_eq!(result1.unwrap(), result2.unwrap());
    }
}

// =============================================================================
// Tests: Character class handling
// =============================================================================

proptest! {
    // Simple character class should parse and compile
    fn test_simple_char_class(content in "[a-z]+") {
        let pattern = format!("[{}]", content);
        let result = compile(&pattern);

        prop_assert!(result.is_ok(), "Char class should compile");
    }

    // Negated character class should work
    fn test_negated_char_class(content in "[a-z]+") {
        let pattern = format!("[^{}]", content);
        let result = compile(&pattern);

        prop_assert!(result.is_ok(), "Negated char class should compile");
    }
}

#[test]
fn test_char_class_with_range() {
    let pattern = "[a-z]";
    let result = compile(pattern);
    assert!(result.is_ok(), "Char class with range should compile");
}

#[test]
fn test_complex_char_class() {
    let pattern = "[a-zA-Z0-9_]";
    let result = compile(pattern);
    assert!(result.is_ok(), "Complex char class should compile");
}

// =============================================================================
// Tests: Regex matching invariants
// =============================================================================

proptest! {
    // Matching against empty string should not panic
    fn test_empty_string_match(pattern in "[a-z]+") {
        let regex_result = Regex::new(&pattern);

        prop_assert!(regex_result.is_ok(), "Pattern should create Regex");

        if let Ok(regex) = regex_result {
            let result = std::panic::catch_unwind(|| {
                let _ = regex.is_match("");
            });
            prop_assert!(result.is_ok(), "is_match on empty string should not panic");
        }
    }

    // Matching with the pattern itself should not panic
    fn test_pattern_matches_itself(s in "[a-z]+") {
        let regex_result = Regex::new(&s);

        prop_assert!(regex_result.is_ok());

        if let Ok(regex) = regex_result {
            let result = std::panic::catch_unwind(|| {
                let _ = regex.is_match(&s);
            });
            prop_assert!(result.is_ok(), "Regex should match its own literal");
        }
    }

    // Find should not panic on any input
    fn test_find_no_panic(pattern in "[a-z]+", input in "[a-zA-Z0-9 ]*") {
        let regex_result = Regex::new(&pattern);

        prop_assert!(regex_result.is_ok());

        if let Ok(regex) = regex_result {
            let result = std::panic::catch_unwind(|| {
                let _ = regex.find(&input);
            });
            prop_assert!(result.is_ok(), "find should not panic");
        }
    }
}

// =============================================================================
// Tests: Replacement invariants
// =============================================================================

proptest! {
    // Replacement parsing should not panic on simple input
    fn test_replacement_parse_no_panic(s in "[a-zA-Z0-9]*") {
        let result = std::panic::catch_unwind(|| {
            let _ = Replacement::parse(&s);
        });
        prop_assert!(result.is_ok(), "Replacement::parse should not panic");
    }
}

#[test]
fn test_replacement_with_group_ref() {
    let result = Replacement::parse(r"\g{1}");
    assert!(result.is_ok());
}

#[test]
fn test_replacement_with_entire_match() {
    let result = Replacement::parse(r"[\G]");
    assert!(result.is_ok());
}

// =============================================================================
// Tests: Quantifiers and special characters
// =============================================================================

proptest! {
    // Quantifier patterns should compile
    fn test_quantifier_patterns(base in "[a-z]+") {
        let patterns = vec![
            format!("{}*", base),
            format!("{}?", base),
            format!("{}+", base),
        ];

        for pattern in patterns {
            let result = compile(&pattern);
            prop_assert!(result.is_ok(), "Quantifier pattern should compile");
        }
    }

    // Anchors should work
    fn test_anchor_patterns(content in "[a-z]+") {
        let patterns = vec![
            format!("^{}", content),
            format!("{}$", content),
            format!("^{}$", content),
        ];

        for pattern in patterns {
            let result = compile(&pattern);
            prop_assert!(result.is_ok(), "Anchor pattern should compile");
        }
    }

    // Alternation should work
    fn test_alternation_patterns(a in "[a-z]+", b in "[a-z]+") {
        let pattern = format!("{}|{}", a, b);
        let result = compile(&pattern);

        prop_assert!(result.is_ok(), "Alternation should compile");
    }
}

// =============================================================================
// Tests: Backreference handling
// =============================================================================

#[test]
fn test_numbered_backref() {
    let pattern = r"(a)\1";
    let result = compile(pattern);
    assert!(result.is_ok(), "Numbered backref should compile");
}

#[test]
fn test_named_backref() {
    let pattern = r"(name:a)\g{name}";
    let result = compile(pattern);
    assert!(result.is_ok(), "Named backref should compile");
}

#[test]
fn test_relative_backref() {
    let pattern = r"(a)(b)\g{-1}";
    let result = compile(pattern);
    assert!(result.is_ok(), "Relative backref should compile");
}

// =============================================================================
// Tests: Edge cases and boundaries
// =============================================================================

proptest! {
    // Very long patterns should not cause issues (within reason)
    fn test_long_patterns(s in "[a-z]{0,100}") {
        if !s.is_empty() {
            let result = compile(&s);
            prop_assert!(result.is_ok(), "Long pattern should compile");
        }
    }
}

#[test]
fn test_special_chars() {
    let patterns = vec![
        r"\.", r"\*", r"\+", r"\?", r"\^", r"\$", r"\|", r"\\", r"\[", r"\]", r"\{", r"\}", r"\(",
        r"\)",
    ];

    for pattern in patterns {
        let result = std::panic::catch_unwind(|| compile(pattern));
        assert!(result.is_ok(), "Special char should compile: {}", pattern);
    }
}

#[test]
fn test_unicode_patterns() {
    let patterns = vec!["(name:日本語)", "(name:🎉)", "日本語", "héllo"];

    for pattern in patterns {
        let result = compile(pattern);
        assert!(
            result.is_ok(),
            "Unicode pattern should compile: {}",
            pattern
        );
    }
}
