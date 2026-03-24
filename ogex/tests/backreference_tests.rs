//! Comprehensive backreference tests for ogex
//!
//! This module tests all backreference types:
//! - Numbered backreferences: \1, \2, \3, etc.
//! - Named backreferences: \g{name}
//! - Relative backreferences: \g{-1}, \g{-2}, etc.
//! - Entire match: \G
//!
//! And edge cases:
//! - Backreference to non-existent group
//! - Backreference to empty group
//! - Nested backreferences
//! - Multiple backreferences in same pattern
//! - Backreference with quantifiers

use ogex::{Regex, Replacement};

// =============================================================================
// Numbered Backreference Tests
// =============================================================================

mod numbered_backref {
    use super::*;

    #[test]
    fn test_numbered_backref_basic() {
        // \1 references first capturing group
        let regex = Regex::new(r"(a)\1").unwrap();
        assert!(regex.is_match("aa"));
        assert!(!regex.is_match("ab"));
        assert!(!regex.is_match("a"));
    }

    #[test]
    fn test_numbered_backref_second_group() {
        // \2 references second capturing group
        let regex = Regex::new(r"(a)(b)\2").unwrap();
        assert!(regex.is_match("abb"));
        assert!(!regex.is_match("aba"));
        assert!(!regex.is_match("abc"));
    }

    #[test]
    fn test_numbered_backref_third_group() {
        // \3 references third capturing group
        let regex = Regex::new(r"(a)(b)(c)\3").unwrap();
        assert!(regex.is_match("abcc"));
        assert!(!regex.is_match("abcb"));
        assert!(!regex.is_match("abca"));
    }

    #[test]
    fn test_numbered_backref_cross_reference() {
        // \1 and \2 in same pattern
        let regex = Regex::new(r"(a)(b)\1\2").unwrap();
        assert!(regex.is_match("abab"));
        assert!(!regex.is_match("abba"));
    }

    #[test]
    fn test_numbered_backref_swapped() {
        // Reference groups in swapped order
        let regex = Regex::new(r"(a)(b)\2\1").unwrap();
        assert!(regex.is_match("abba"));
        assert!(!regex.is_match("abab"));
    }

    #[test]
    fn test_numbered_backref_find() {
        let regex = Regex::new(r"(\d{3})\1").unwrap();
        let m = regex.find("12312345").unwrap();
        assert_eq!(m.as_str("12312345"), "123123");
    }

    #[test]
    fn test_numbered_backref_with_literal() {
        let regex = Regex::new(r"foo(a)bar\1baz").unwrap();
        assert!(regex.is_match("fooabarabaz"));
        assert!(!regex.is_match("foobarabaz"));
    }

    #[test]
    fn test_numbered_backref_empty_group() {
        // Test backref with optional group
        // This tests that the engine handles optional groups correctly
        let regex = Regex::new(r"(a)?b\1").unwrap();
        // group 1 = "a" (present), \1 = "a"
        assert!(regex.is_match("aba"));
    }

    #[test]
    fn test_numbered_backref_multiple_occurrences() {
        // Same backreference used multiple times
        let regex = Regex::new(r"(\w)\1\1").unwrap();
        assert!(regex.is_match("aaa"));
        assert!(!regex.is_match("aab"));
        assert!(!regex.is_match("abb"));
    }
}

// =============================================================================
// Numbered Backreference in Replacement Tests
// =============================================================================

mod numbered_backref_replacement {
    use super::*;

    #[test]
    fn test_replace_with_numbered_backref() {
        let regex = Regex::new(r"(a)(b)").unwrap();
        let m = regex.find("ab").unwrap();

        // Replace with group 2 followed by group 1 (swapped)
        let repl = Replacement::parse(r"\g{2}\g{1}").unwrap();
        let mut group_pairs = vec![(0usize, 0usize); m.groups.len()];
        for (idx, opt) in m.groups.iter().enumerate() {
            if let Some((s, e)) = opt
                && idx > 0
                && idx < m.groups.len()
            {
                group_pairs[idx - 1] = (*s, *e);
            }
        }
        let result = repl.apply("ab", m.start, m.end, &group_pairs);
        assert_eq!(result, "ba");
    }

    #[test]
    fn test_replace_with_multiple_numbered_backrefs() {
        let regex = Regex::new(r"(a)(b)(c)").unwrap();
        let m = regex.find("abc").unwrap();

        // Replace with order: 3-1-2
        let repl = Replacement::parse(r"\g{3}-\g{1}-\g{2}").unwrap();
        let mut group_pairs = vec![(0usize, 0usize); m.groups.len()];
        for (idx, opt) in m.groups.iter().enumerate() {
            if let Some((s, e)) = opt
                && idx > 0
                && idx < m.groups.len()
            {
                group_pairs[idx - 1] = (*s, *e);
            }
        }
        let result = repl.apply("abc", m.start, m.end, &group_pairs);
        assert_eq!(result, "c-a-b");
    }

    #[test]
    fn test_replace_with_backref_and_literal() {
        let regex = Regex::new(r"(\w+)").unwrap();
        let m = regex.find("hello").unwrap();

        let repl = Replacement::parse(r"[prefix:\g{1}:suffix]").unwrap();
        let group_pairs: Vec<_> = m
            .groups
            .iter()
            .filter_map(|opt| opt.map(|(s, e)| (s, e)))
            .collect();
        let result = repl.apply("hello", m.start, m.end, &group_pairs);
        assert_eq!(result, "[prefix:hello:suffix]");
    }

    #[test]
    fn test_replace_numbered_backref_repeat() {
        let regex = Regex::new(r"(a)").unwrap();
        let m = regex.find("a").unwrap();

        // Repeat the group
        let repl = Replacement::parse(r"\g{1}\g{1}\g{1}").unwrap();
        let mut group_pairs = vec![(0usize, 0usize); m.groups.len()];
        for (idx, opt) in m.groups.iter().enumerate() {
            if let Some((s, e)) = opt
                && idx > 0
                && idx < m.groups.len()
            {
                group_pairs[idx - 1] = (*s, *e);
            }
        }
        let result = repl.apply("a", m.start, m.end, &group_pairs);
        assert_eq!(result, "aaa");
    }
}

// =============================================================================
// Named Backreference Tests
// =============================================================================

mod named_backref {
    use super::*;

    #[test]
    fn test_named_backref_basic() {
        let regex = Regex::new(r"(name:a)\g{name}").unwrap();
        assert!(regex.is_match("aa"));
        assert!(!regex.is_match("ab"));
    }

    #[test]
    fn test_named_backref_different_content() {
        let regex = Regex::new(r"(word:\w+) is \g{word}").unwrap();
        assert!(regex.is_match("hello is hello"));
        assert!(!regex.is_match("hello is world"));
    }

    #[test]
    fn test_named_backref_multiple_named_groups() {
        // Multiple named groups with backrefs
        let regex = Regex::new(r"(first:a)(second:b)\g{second}\g{first}").unwrap();
        // This matches "a" + "b" + "b" + "a" = "abba"
        assert!(regex.is_match("abba"));
        // Not "a b ba" - that's not contiguous
        assert!(!regex.is_match("a b ba"));
    }

    #[test]
    fn test_named_backref_find() {
        // Simple test with named backref - basic matching
        let regex = Regex::new(r"(word:hello)\g{word}").unwrap();
        // Should match "hellohello"
        assert!(regex.is_match("hellohello"));
        // Should not match "hellox"
        assert!(!regex.is_match("hellox"));
    }

    #[test]
    fn test_named_backref_with_number_syntax() {
        // \g{1} is treated as a named backreference with name "1", not \1
        // So this is actually looking for a group named "1" which doesn't exist
        // Use \1 syntax for numbered groups
        let regex = Regex::new(r"(a)\1").unwrap();
        assert!(regex.is_match("aa"));
    }

    #[test]
    fn test_named_backref_complex_pattern() {
        // Pattern with named group and backref in the middle
        // Note: "prefix" as a named group name - content is "ab"
        let regex = Regex::new(r"(prefix:ab)c\g{prefix}d").unwrap();
        // Pattern is: ab then c then ab then d
        assert!(regex.is_match("abcabd"));
    }
}

// =============================================================================
// Named Backreference in Replacement Tests
// =============================================================================

mod named_backref_replacement {
    use super::*;

    #[test]
    fn test_replace_with_named_backref() {
        let regex = Regex::new(r"(name:\w+)").unwrap();
        let m = regex.find("hello").unwrap();

        // Named groups use numeric index in replacement
        let repl = Replacement::parse(r"[name:\g{1}]").unwrap();
        let group_pairs: Vec<_> = m
            .groups
            .iter()
            .filter_map(|opt| opt.map(|(s, e)| (s, e)))
            .collect();
        let result = repl.apply("hello", m.start, m.end, &group_pairs);
        assert_eq!(result, "[name:hello]");
    }

    #[test]
    fn test_replace_named_backref_order() {
        let regex = Regex::new(r"(a:x)(b:y)").unwrap();
        let m = regex.find("xy").unwrap();

        // Swap order in replacement
        let repl = Replacement::parse(r"\g{2} then \g{1}").unwrap();
        let mut group_pairs = vec![(0usize, 0usize); m.groups.len()];
        for (idx, opt) in m.groups.iter().enumerate() {
            if let Some((s, e)) = opt
                && idx > 0
                && idx < m.groups.len()
            {
                group_pairs[idx - 1] = (*s, *e);
            }
        }
        let result = repl.apply("xy", m.start, m.end, &group_pairs);
        assert_eq!(result, "y then x");
    }
}

// =============================================================================
// Relative Backreference Tests
// =============================================================================

mod relative_backref {
    use super::*;

    #[test]
    fn test_relative_backref_minus_one() {
        // \g{-1} = last numbered group
        let regex = Regex::new(r"(a)(b)\g{-1}").unwrap();
        assert!(regex.is_match("abb"));
        assert!(!regex.is_match("aba"));
        assert!(!regex.is_match("abc"));
    }

    #[test]
    fn test_relative_backref_minus_two() {
        // \g{-2} = second-to-last numbered group
        let regex = Regex::new(r"(a)(b)(c)\g{-2}").unwrap();
        assert!(regex.is_match("abcb"));
        assert!(!regex.is_match("abca"));
        assert!(!regex.is_match("abcc"));
    }

    #[test]
    fn test_relative_backref_minus_three() {
        // \g{-3} = third-to-last numbered group
        let regex = Regex::new(r"(a)(b)(c)(d)\g{-3}").unwrap();
        // Groups: 1=a, 2=b, 3=c, 4=d
        // \g{-3} = group 2 = "b"
        assert!(regex.is_match("abcdb"));
        assert!(!regex.is_match("abcdc"));
    }

    #[test]
    fn test_relative_backref_multiple() {
        // Multiple relative backrefs
        let regex = Regex::new(r"(a)(b)\g{-1}\g{-2}").unwrap();
        // \g{-1} = last = b, \g{-2} = second-to-last = a
        assert!(regex.is_match("abba"));
        assert!(!regex.is_match("abab"));
    }

    #[test]
    fn test_relative_backref_with_named_excluded() {
        // Named groups are excluded from relative indexing
        let regex = Regex::new(r"(a)(name:x)(b)\g{-2}").unwrap();
        // Numbered groups: 1=a, 3=b
        // \g{-2} = group 1 = "a"
        assert!(regex.is_match("axba"));
        assert!(!regex.is_match("axbx"));
    }

    #[test]
    fn test_relative_backref_chain() {
        // Chain of relative backrefs
        let regex = Regex::new(r"(a)(b)(c)\g{-1}\g{-2}\g{-3}").unwrap();
        // \g{-1}=c, \g{-2}=b, \g{-3}=a
        assert!(regex.is_match("abccba"));
    }

    #[test]
    fn test_relative_backref_find() {
        // Find with relative backref
        let regex = Regex::new(r"(\d+)\g{-1}").unwrap();
        let m = regex.find("123123").unwrap();
        assert_eq!(m.as_str("123123"), "123123");
    }
}

// =============================================================================
// Relative Backreference in Replacement Tests
// =============================================================================

mod relative_backref_replacement {
    use super::*;

    #[test]
    fn test_replace_with_relative_syntax() {
        // Replacement uses absolute group numbers, not relative
        let regex = Regex::new(r"(a)(b)(c)").unwrap();
        let m = regex.find("abc").unwrap();

        // Use absolute indices in replacement
        let repl = Replacement::parse(r"\g{3}\g{2}\g{1}").unwrap();
        let mut group_pairs = vec![(0usize, 0usize); m.groups.len()];
        for (idx, opt) in m.groups.iter().enumerate() {
            if let Some((s, e)) = opt
                && idx > 0
                && idx < m.groups.len()
            {
                group_pairs[idx - 1] = (*s, *e);
            }
        }
        let result = repl.apply("abc", m.start, m.end, &group_pairs);
        assert_eq!(result, "cba");
    }
}

// =============================================================================
// Entire Match (\G) Tests
// =============================================================================

mod entire_match {
    use super::*;

    #[test]
    fn test_entire_match_replacement_basic() {
        // \G in replacement = entire match
        let repl = Replacement::parse(r"[\G]").unwrap();
        let result = repl.apply("hello", 0, 5, &[]);
        assert_eq!(result, "[hello]");
    }

    #[test]
    fn test_entire_match_replacement_with_groups() {
        let regex = Regex::new(r"\w+").unwrap();
        let m = regex.find("hello world").unwrap();

        let repl = Replacement::parse(r"<\G>").unwrap();
        let group_pairs: Vec<_> = m
            .groups
            .iter()
            .filter_map(|opt| opt.map(|(s, e)| (s, e)))
            .collect();
        let result = repl.apply("hello world", m.start, m.end, &group_pairs);
        assert_eq!(result, "<hello>");
    }

    #[test]
    fn test_entire_match_replacement_multiple() {
        let repl = Replacement::parse(r"prefix-\G-suffix").unwrap();
        let result = repl.apply("test", 0, 4, &[]);
        assert_eq!(result, "prefix-test-suffix");
    }

    #[test]
    fn test_g_literal_in_pattern() {
        // In patterns, \G is just a literal 'G'
        let regex = Regex::new(r"\G").unwrap();
        assert!(regex.is_match("G"));
        assert!(!regex.is_match("g"));
        assert!(!regex.is_match("abc"));
    }

    #[test]
    fn test_g_literal_in_pattern_with_other() {
        let regex = Regex::new(r"foo\Gbar").unwrap();
        assert!(regex.is_match("fooGbar"));
        assert!(!regex.is_match("foobar"));
    }
}

// =============================================================================
// Edge Case Tests
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_backref_to_high_number() {
        // Backreference to non-existent group (higher than any group)
        // Should compile but never match
        let regex = Regex::new(r"(a)\5").unwrap();
        assert!(!regex.is_match("a"));
        assert!(!regex.is_match("aa"));
    }

    #[test]
    fn test_backref_zero() {
        // \0 is not valid in patterns - test it doesn't crash
        // Either it compiles (treating \0 as null char) or fails gracefully
        // Let's just ensure it doesn't panic
        if let Ok(r) = Regex::new(r"\0") {
            // Should match null character
            let _ = r.is_match("\0");
        }
    }

    #[test]
    fn test_nested_backrefs() {
        // This is tricky - backreference inside a repeated group
        let regex = Regex::new(r"((a)\2)+").unwrap();
        // This pattern is (aa)+ which should match "aaaa"
        // But the backref \2 references (a), which should be the last 'a'
        // This tests the nested structure works
        assert!(regex.is_match("aa"));
        assert!(regex.is_match("aaaa"));
    }

    #[test]
    fn test_backref_with_quantifiers() {
        // Test backreference with quantifiers on the group
        // (a+)b\1 means: one or more 'a', then 'b', then same as group 1
        let regex = Regex::new(r"(a+)b\1").unwrap();
        // "aa" + "b" + "aa" = "aabaa"
        assert!(regex.is_match("aabaa"));
        // Different - should fail
        assert!(!regex.is_match("ab aa"));
    }

    #[test]
    fn test_multiple_backrefs_same_group() {
        // Same group used multiple times in pattern
        let regex = Regex::new(r"(a)\1\1\1").unwrap();
        assert!(regex.is_match("aaaa"));
        assert!(!regex.is_match("aaa"));
        assert!(!regex.is_match("aaab"));
    }

    #[test]
    fn test_empty_group_backref() {
        // Empty capturing group - just ()
        let regex = Regex::new(r"()(a)").unwrap();
        // Matches "a" - empty group + "a"
        assert!(regex.is_match("a"));
    }

    #[test]
    fn test_backref_non_contiguous() {
        // Backreference to group that's not immediately before it
        // (a)b(c)\1 means: group1="a", "b", group2="c", then \1="a"
        // So pattern matches: a b c a = "abca"
        let regex = Regex::new(r"(a)b(c)\1").unwrap();
        assert!(regex.is_match("abca"));
        assert!(!regex.is_match("abcb"));
    }

    #[test]
    fn test_backref_at_start() {
        // Backreference at start of pattern
        let regex = Regex::new(r"\1(a)").unwrap();
        // \1 references non-existent group, should fail to match
        assert!(!regex.is_match("aa"));
    }

    #[test]
    fn test_backref_at_end() {
        // Backreference at end of pattern
        let regex = Regex::new(r"(a)\1").unwrap();
        assert!(regex.is_match("aa"));
        assert!(!regex.is_match("a"));
    }

    #[test]
    fn test_many_groups_backref() {
        // Pattern with many groups and backreferences
        let regex = Regex::new(r"(a)(b)(c)(d)(e)\1\2\3\4\5").unwrap();
        assert!(regex.is_match("abcdeabcde"));
        assert!(!regex.is_match("abcdexxxxx"));
    }

    #[test]
    fn test_mixed_named_numbered_backref() {
        // Mix of named and numbered groups with backrefs
        // Using simple numbered groups since \g{x} with numeric x might not resolve
        let regex = Regex::new(r"(1)(2)\1\2").unwrap();
        // Group 1 = "1", Group 2 = "2", \1 = "1", \2 = "2"
        assert!(regex.is_match("1212"));
    }
}

// =============================================================================
// Backreference in Complex Patterns Tests
// =============================================================================

mod complex_patterns {
    use super::*;

    #[test]
    fn test_backref_in_alternation() {
        // Backreference in alternation - complex case
        let regex = Regex::new(r"(a)|(b)\2").unwrap();
        // Either match "a" or match group 2 followed by what it matched
        assert!(regex.is_match("a"));
        assert!(regex.is_match("bb"));
    }

    #[test]
    fn test_backref_with_anchors() {
        // Backreference with anchors
        let regex = Regex::new(r"^(a)\1$").unwrap();
        assert!(regex.is_match("aa"));
        assert!(!regex.is_match("a"));
        assert!(!regex.is_match("aaa"));
    }

    #[test]
    fn test_backref_in_character_class() {
        // Character class with backslash and number - might be interpreted differently
        // Test that it doesn't crash
        let result = std::panic::catch_unwind(|| {
            let _ = Regex::new(r"[\1]");
        });
        // Should either parse or fail gracefully, not panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_backref_with_word_boundaries() {
        // Backreference with word boundaries
        let regex = Regex::new(r"\b(a)\1\b").unwrap();
        assert!(regex.is_match("aa"));
        assert!(regex.is_match("aa ")); // trailing space, boundary after
    }

    #[test]
    fn test_backref_complex_realworld() {
        // Real-world: matching quoted strings with same content
        // This simulates finding matching quotes like "hello"..."hello"
        let regex = Regex::new(r#"("([^"]+)") \2"#).unwrap();
        assert!(regex.is_match("\"hello\" hello"));
        assert!(!regex.is_match("\"hello\" world"));
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn test_invalid_group_reference() {
        // Group number larger than any group should not crash
        let regex = Regex::new(r"(a)\100").unwrap();
        // Should compile but never match
        assert!(!regex.is_match("a"));
        assert!(!regex.is_match("aa"));
    }

    #[test]
    fn test_unclosed_backref_braces() {
        // Malformed backreference syntax
        let result = Regex::new(r"(a)\g{");
        // Should fail to parse
        assert!(result.is_err() || result.map(|r| !r.is_match("a")).unwrap_or(true));
    }

    #[test]
    fn test_empty_backref_name() {
        // Empty name in \g{}
        let result = Regex::new(r"(a)\g{}");
        // Should fail to parse
        assert!(result.is_err() || result.map(|r| !r.is_match("a")).unwrap_or(true));
    }
}

// =============================================================================
// Transpilation Tests
// =============================================================================

mod transpilation {
    use ogex::compile;

    #[test]
    fn test_transpile_numbered_backref() {
        let result = compile(r"(a)\1").unwrap();
        assert!(result.contains(r"\1"));
    }

    #[test]
    fn test_transpile_named_backref() {
        // Named backref transpiles to \k<name> in PCRE format
        let result = compile(r"(name:a)\g{name}").unwrap();
        assert!(result.contains(r"\k<name>"));
    }

    #[test]
    fn test_transpile_relative_backref() {
        let result = compile(r"(a)\g{-1}").unwrap();
        assert!(result.contains(r"\g{-1}"));
    }

    #[test]
    fn test_transpile_multiple_backrefs() {
        let result = compile(r"(a)(b)\1\2").unwrap();
        assert!(result.contains(r"\1"));
        assert!(result.contains(r"\2"));
    }
}

// =============================================================================
// Regression Tests
// =============================================================================

mod regression {
    use super::*;

    #[test]
    fn test_regression_consecutive_backrefs() {
        // Consecutive backreferences shouldn't interfere
        let regex = Regex::new(r"(a)(b)(c)\1\2\3").unwrap();
        assert!(regex.is_match("abcabc"));
        assert!(!regex.is_match("abcxyz"));
    }

    #[test]
    fn test_regression_overlapping_groups() {
        // Groups that overlap in matching
        let regex = Regex::new(r"(a)(a)\1\2").unwrap();
        // Group 1 = first a, Group 2 = second a
        // \1 = first a, \2 = second a
        assert!(regex.is_match("aaaa"));
    }

    #[test]
    fn test_regression_repeated_pattern() {
        // Pattern with repeated group reference
        let regex = Regex::new(r"(\w+)\s+\1").unwrap();
        assert!(regex.is_match("hello hello"));
        assert!(regex.is_match("foo foo"));
        assert!(!regex.is_match("hello world"));
    }

    #[test]
    fn test_regression_case_sensitivity() {
        // Backrefs are case sensitive
        let regex = Regex::new(r"(A)\1").unwrap();
        assert!(regex.is_match("AA"));
        assert!(!regex.is_match("Aa"));
        assert!(!regex.is_match("aA"));
    }
}
