//! Pathological pattern tests for ogex
//!
//! These tests verify that the regex engine handles patterns known to cause
//! catastrophic backtracking in naive implementations. The ogex engine uses
//! memoization and epsilon closure caching to handle these patterns reasonably.
//!
//! Test categories:
//! - Nested quantifiers: (a+)+, (a*)*, (a?)*
//! - Alternation overlap: (a|a|a)+
//! - Backtracking bombs: (a+)*b
//! - Catastrophic backtracking patterns
//! - Large inputs with timeouts

use ogex::Regex;
use std::time::{Duration, Instant};

// =============================================================================
// Helper Functions
// =============================================================================

/// Run a test with a timeout to prevent hanging
fn test_with_timeout<F>(pattern: &str, input: &str, timeout_ms: u64, f: F)
where
    F: FnOnce(&Regex, &str),
{
    let start = Instant::now();
    let regex = Regex::new(pattern).expect("Failed to compile pattern");
    let duration = start.elapsed();

    // Compilation should be fast
    assert!(
        duration < Duration::from_millis(100),
        "Pattern compilation took too long: {:?}",
        duration
    );

    let start = Instant::now();
    f(&regex, input);
    let duration = start.elapsed();

    assert!(
        duration < Duration::from_millis(timeout_ms),
        "Pattern matching timed out after {:?} (limit: {}ms): pattern='{}' input='{}'",
        duration,
        timeout_ms,
        pattern,
        input
    );
}

/// Test that pattern compilation succeeds (doesn't hang)
fn test_compile_only(pattern: &str) {
    let start = Instant::now();
    let regex = Regex::new(pattern);
    let duration = start.elapsed();

    assert!(regex.is_ok(), "Failed to compile pattern: {}", pattern);
    assert!(
        duration < Duration::from_millis(100),
        "Pattern compilation took too long: {:?}: {}",
        duration,
        pattern
    );
}

// =============================================================================
// Nested Quantifier Tests
// =============================================================================

mod nested_quantifiers {
    use super::*;

    #[test]
    fn test_nested_plus_quantifier() {
        // (a+)+ - nested plus quantifier
        test_compile_only("(a+)+");

        // Short input should match quickly
        test_with_timeout("(a+)+", "aaa", 1000, |regex, input| {
            assert!(regex.is_match(input), "Should match 'aaa'");
        });

        // Slightly longer but still reasonable
        test_with_timeout("(a+)+", "aaaaaa", 1000, |regex, input| {
            assert!(regex.is_match(input), "Should match 'aaaaaa'");
        });
    }

    #[test]
    fn test_nested_star_quantifier() {
        // (a*)* - nested star quantifier (matches empty string)
        test_compile_only("(a*)*");

        test_with_timeout("(a*)", "", 1000, |regex, input| {
            assert!(regex.is_match(input), "Should match empty string");
        });

        test_with_timeout("(a*)*", "aaa", 1000, |regex, input| {
            assert!(regex.is_match(input), "Should match 'aaa'");
        });
    }

    #[test]
    fn test_nested_question_quantifier() {
        // (a?)* - nested question mark quantifier
        test_compile_only("(a?)*");

        test_with_timeout("(a?)*", "", 1000, |regex, input| {
            assert!(regex.is_match(input), "Should match empty string");
        });

        test_with_timeout("(a?)*", "aaa", 1000, |regex, input| {
            assert!(regex.is_match(input), "Should match 'aaa'");
        });
    }

    #[test]
    fn test_deeply_nested_quantifiers() {
        // ((a+)+)+ - deeply nested
        test_compile_only("((a+)+)+");

        test_with_timeout("((a+)+)+", "aaa", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }

    #[test]
    fn test_nested_quantifier_with_different_chars() {
        // (a+b+)+ - nested with multiple chars
        test_compile_only("(a+b+)+");

        test_with_timeout("(a+b+)+", "abab", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }

    #[test]
    fn test_nested_quantifier_complex() {
        // (.*)* - very dangerous with any input
        test_compile_only("(.*)*");

        test_with_timeout("(.*)*", "abc", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }
}

// =============================================================================
// Alternation Overlap Tests
// =============================================================================

mod alternation_overlap {
    use super::*;

    #[test]
    fn test_simple_alternation() {
        // (a|a|a)+ - overlapping alternatives
        test_compile_only("(a|a|a)+");

        test_with_timeout("(a|a|a)+", "aaa", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });

        test_with_timeout("(a|a|a)+", "a", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }

    #[test]
    fn test_alternation_with_quantifier() {
        // (a|b|c)* - alternation with star
        test_compile_only("(a|b|c)*");

        test_with_timeout("(a|b|c)*", "abcabc", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });

        test_with_timeout("(a|b|c)*", "", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }

    #[test]
    fn test_overlapping_alternation_nested() {
        // ((a|b)|(a|b)|(a|b))+
        test_compile_only("((a|b)|(a|b)|(a|b))+");

        test_with_timeout("((a|b)|(a|b)|(a|b))+", "abab", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }

    #[test]
    fn test_alternation_many_options() {
        // (a|a|a|a|a|a|a|a|a|a)+ - many same options
        test_compile_only("(a|a|a|a|a|a|a|a|a|a)+");

        test_with_timeout(
            "(a|a|a|a|a|a|a|a|a|a)+",
            "aaaaaaaaaa",
            1000,
            |regex, input| {
                assert!(regex.is_match(input));
            },
        );
    }
}

// =============================================================================
// Backtracking Bomb Tests
// =============================================================================

mod backtracking_bombs {
    use super::*;

    #[test]
    fn test_plus_star_bomb() {
        // (a+)*b - classic backtracking bomb when 'b' doesn't match
        test_compile_only("(a+)*b");

        // Should fail quickly when 'b' doesn't exist in input
        test_with_timeout("(a+)*b", "aaaaaaaa", 1000, |regex, input| {
            assert!(!regex.is_match(input), "Should NOT match 'aaaaaaaa'");
        });

        // Should match when 'b' exists
        test_with_timeout("(a+)*b", "aaaaaaab", 1000, |regex, input| {
            assert!(regex.is_match(input), "Should match 'aaaaaaab'");
        });
    }

    #[test]
    fn test_star_plus_bomb() {
        // (a*)*b - another dangerous pattern
        test_compile_only("(a*)*b");

        test_with_timeout("(a*)*b", "aaaaaaaa", 1000, |regex, input| {
            assert!(!regex.is_match(input));
        });
    }

    #[test]
    fn test_question_star_bomb() {
        // (a?)*b
        test_compile_only("(a?)*b");

        test_with_timeout("(a?)*b", "aaaa", 1000, |regex, input| {
            assert!(!regex.is_match(input));
        });
    }

    #[test]
    fn test_nested_bomb() {
        // ((a+)*)*b
        test_compile_only("((a+)*)*b");

        test_with_timeout("((a+)*)*b", "aaaaaaaa", 1000, |regex, input| {
            assert!(!regex.is_match(input));
        });
    }
}

// =============================================================================
// Catastrophic Backtracking Patterns
// =============================================================================

mod catastrophic_patterns {
    use super::*;

    #[test]
    fn test_aa_star_bomb() {
        // (a+)* matches "a" but fails on "b" - exponential in some engines
        test_compile_only("(a+)*");

        test_with_timeout("(a+)*", "aaa", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }

    #[test]
    fn test_optional_nested() {
        // (a?b)+ - pattern that can cause issues
        test_compile_only("(a?b)+");

        test_with_timeout("(a?b)+", "bbbb", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }

    #[test]
    fn test_alternation_quantifier_overlap() {
        // (a|a)* - same as (a*) but with alternation overhead
        test_compile_only("(a|a)*");

        test_with_timeout("(a|a)*", "aaaaa", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }

    #[test]
    fn test_complex_nested() {
        // ((a+)?)* - optional inside nested star
        test_compile_only("((a+)?)*");

        test_with_timeout("((a+)?)*", "aaa", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }
}

// =============================================================================
// Large Input Tests with Timeout
// =============================================================================

mod large_input_tests {
    use super::*;

    #[test]
    fn test_large_matching_input() {
        // Pattern that matches, should complete quickly
        let input = "a".repeat(100);
        test_with_timeout("a+", &input, 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }

    #[test]
    fn test_large_non_matching_input() {
        // Pattern that doesn't match, should fail fast (not timeout)
        let input = "b".repeat(100);
        test_with_timeout("a+", &input, 1000, |regex, input| {
            assert!(!regex.is_match(input));
        });
    }

    #[test]
    fn test_large_input_with_nested_quantifier() {
        // Nested quantifier with large matching input
        let input = "a".repeat(50);
        test_with_timeout("(a+)+", &input, 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }

    #[test]
    fn test_large_input_backtracking_bomb() {
        // Backtracking bomb with non-matching input
        let input = "a".repeat(50);
        test_with_timeout("(a+)*b", &input, 1000, |regex, input| {
            assert!(!regex.is_match(input));
        });
    }

    #[test]
    fn test_medium_alternation_input() {
        let input = "abcabcabcabc";
        test_with_timeout("(a|b|c)+", &input, 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }
}

// =============================================================================
// Edge Cases
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_empty_input_nested() {
        test_with_timeout("(a+)*", "", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }

    #[test]
    fn test_empty_input_alternation() {
        test_with_timeout("(a|b)*", "", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }

    #[test]
    fn test_single_char_nested() {
        test_with_timeout("(a+)+", "a", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }

    #[test]
    fn test_two_char_nested() {
        test_with_timeout("(a+)+", "aa", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }

    #[test]
    fn test_find_with_nested_quantifier() {
        // Test that find() also works without hanging
        let regex = Regex::new("(a+)+").unwrap();
        let input = "xxaaaaay";

        let start = Instant::now();
        let result = regex.find(input);
        let duration = start.elapsed();

        assert!(duration < Duration::from_millis(1000), "find() timed out");
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_str(input), "aaaaa");
    }
}

// =============================================================================
// Regression Tests
// =============================================================================

mod regression {
    use super::*;

    #[test]
    fn test_previously_slow_patterns() {
        // These patterns were historically slow in naive implementations

        // Pattern: (a+)* - exponential in backtracking engines
        test_with_timeout("(a+)*", "aaa", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });

        // Pattern: (a*)* - similar issue
        test_with_timeout("(a*)*", "aaa", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }

    #[test]
    fn test_quantifier_on_quantifier() {
        // (a{1,2}){1,2} - quantifier on quantified group
        test_compile_only("(a{1,2}){1,2}");

        test_with_timeout("(a{1,2}){1,2}", "aa", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });

        test_with_timeout("(a{1,2}){1,2}", "aaa", 1000, |regex, input| {
            assert!(regex.is_match(input));
        });
    }
}
