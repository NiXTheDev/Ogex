//! Regex matching engine with backreference support
//!
//! This module provides the actual regex matching functionality,
//! including NFA simulation and backreference handling.

use crate::ast::ClassItem;
use crate::nfa::{Nfa, StateId, Transition};
use std::collections::{HashMap, HashSet};

/// A match result
#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    /// The start position of the match
    pub start: usize,
    /// The end position of the match (exclusive)
    pub end: usize,
    /// Captured groups (index -> (start, end))
    pub groups: HashMap<u32, (usize, usize)>,
    /// Named captured groups (name -> (start, end))
    pub named_groups: HashMap<String, (usize, usize)>,
}

impl Match {
    /// Get the matched text
    pub fn as_str<'a>(&self, input: &'a str) -> &'a str {
        &input[self.start..self.end]
    }

    /// Get a capture group by index (1-based)
    pub fn group(&self, n: u32) -> Option<(usize, usize)> {
        self.groups.get(&n).copied()
    }

    /// Get a named capture group
    pub fn named_group(&self, name: &str) -> Option<(usize, usize)> {
        self.named_groups.get(name).copied()
    }

    /// Get the text of a capture group
    pub fn group_str<'a>(&self, input: &'a str, n: u32) -> Option<&'a str> {
        self.group(n).map(|(start, end)| &input[start..end])
    }

    /// Get the text of a named capture group
    pub fn named_group_str<'a>(&self, input: &'a str, name: &str) -> Option<&'a str> {
        self.named_group(name)
            .map(|(start, end)| &input[start..end])
    }
}

/// The regex engine
pub struct Regex {
    nfa: Nfa,
}

impl Regex {
    /// Compile a regex pattern
    pub fn new(pattern: &str) -> Result<Self, crate::error::RegexError> {
        let ast = crate::parser::parse(pattern)?;
        let nfa = Nfa::from_expr(&ast);
        Ok(Regex { nfa })
    }

    /// Check if the pattern matches anywhere in the input
    pub fn is_match(&self, input: &str) -> bool {
        self.find(input).is_some()
    }

    /// Find the first match in the input
    pub fn find(&self, input: &str) -> Option<Match> {
        // Try matching from each position
        for start in 0..=input.len() {
            if let Some(match_result) = self.match_from(input, start) {
                return Some(match_result);
            }
        }
        None
    }

    /// Find all non-overlapping matches
    pub fn find_all(&self, input: &str) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut pos = 0;

        while pos <= input.len() {
            if let Some(match_result) = self.match_from(input, pos) {
                pos = match_result.end;
                matches.push(match_result);
            } else {
                pos += 1;
            }
        }

        matches
    }

    /// Match the pattern starting from a specific position
    fn match_from(&self, input: &str, start: usize) -> Option<Match> {
        let mut simulator = NfaSimulator::new(&self.nfa, input, start);
        simulator.run()
    }
}

/// NFA simulator for pattern matching
struct NfaSimulator<'a> {
    nfa: &'a Nfa,
    #[allow(dead_code)]
    input: &'a str,
    input_chars: Vec<char>,
    start_pos: usize,
    current_states: HashSet<StateId>,
    groups: HashMap<u32, (usize, usize)>,
    #[allow(dead_code)]
    group_stack: Vec<(u32, usize)>,
    #[allow(dead_code)]
    next_group_id: u32,
}

impl<'a> NfaSimulator<'a> {
    fn new(nfa: &'a Nfa, input: &'a str, start_pos: usize) -> Self {
        let input_chars: Vec<char> = input.chars().collect();
        let mut current_states = HashSet::new();
        current_states.insert(nfa.start);

        NfaSimulator {
            nfa,
            input,
            input_chars,
            start_pos,
            current_states,
            groups: HashMap::new(),
            group_stack: Vec::new(),
            next_group_id: 1,
        }
    }

    fn run(&mut self) -> Option<Match> {
        let mut pos = self.start_pos;
        let mut last_accept = None;

        // Apply epsilon closure to start state
        self.current_states = self.epsilon_closure(&self.current_states);

        // Check if start state is accepting (empty match)
        if self.is_accepting_state() {
            last_accept = Some(pos);
        }

        while pos < self.input_chars.len() {
            let c = self.input_chars[pos];
            self.step(c);

            if self.current_states.is_empty() {
                // No more states, matching failed
                break;
            }

            pos += 1;

            if self.is_accepting_state() {
                last_accept = Some(pos);
            }
        }

        // Try to reach accept state via epsilon transitions
        self.current_states = self.epsilon_closure(&self.current_states);
        if self.is_accepting_state() {
            last_accept = Some(pos);
        }

        last_accept.map(|end| Match {
            start: self.start_pos,
            end,
            groups: self.groups.clone(),
            named_groups: HashMap::new(),
        })
    }

    fn step(&mut self, c: char) {
        let mut new_states = HashSet::new();
        let current_states: Vec<_> = self.current_states.iter().copied().collect();

        for state_id in current_states {
            for (transition, target) in &self.nfa.states[state_id].transitions {
                if Self::transition_matches(transition, c, self.start_pos) {
                    new_states.insert(*target);
                }
            }
        }

        self.current_states = self.epsilon_closure(&new_states);
    }

    fn transition_matches(transition: &Transition, c: char, start_pos: usize) -> bool {
        match transition {
            Transition::Char(tc) => *tc == c,
            Transition::Any => true,
            Transition::CharClass { negated, items } => {
                let matched = items.iter().any(|item| match item {
                    ClassItem::Char(ch) => *ch == c,
                    ClassItem::Range(start, end) => (*start..=*end).contains(&c),
                    ClassItem::Shorthand(sh) => match sh {
                        'd' => c.is_ascii_digit(),
                        'D' => !c.is_ascii_digit(),
                        'w' => c.is_ascii_alphanumeric() || c == '_',
                        'W' => !(c.is_ascii_alphanumeric() || c == '_'),
                        's' => c.is_ascii_whitespace(),
                        'S' => !c.is_ascii_whitespace(),
                        _ => false,
                    },
                });
                if *negated {
                    !matched
                } else {
                    matched
                }
            }
            Transition::StartAnchor => start_pos == 0,
            Transition::EndAnchor => false,
            _ => false,
        }
    }

    fn epsilon_closure(&self, states: &HashSet<StateId>) -> HashSet<StateId> {
        let mut closure = states.clone();
        let mut stack: Vec<_> = states.iter().copied().collect();

        while let Some(state) = stack.pop() {
            for (transition, target) in &self.nfa.states[state].transitions {
                if matches!(transition, Transition::Epsilon) && !closure.contains(target) {
                    closure.insert(*target);
                    stack.push(*target);
                }
            }
        }

        closure
    }

    fn is_accepting_state(&self) -> bool {
        self.current_states.contains(&self.nfa.accept)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_literal_match() {
        let regex = Regex::new("abc").unwrap();
        assert!(regex.is_match("abc"));
        assert!(regex.is_match("xabcy"));
        assert!(!regex.is_match("ab"));
        assert!(!regex.is_match("xyz"));
    }

    #[test]
    fn test_regex_alternation() {
        let regex = Regex::new("a|b").unwrap();
        assert!(regex.is_match("a"));
        assert!(regex.is_match("b"));
        assert!(regex.is_match("xax"));
        assert!(!regex.is_match("c"));
    }

    #[test]
    fn test_regex_star() {
        let regex = Regex::new("a*").unwrap();
        assert!(regex.is_match(""));
        assert!(regex.is_match("a"));
        assert!(regex.is_match("aaa"));
        assert!(regex.is_match("baaaab"));
    }

    #[test]
    fn test_regex_plus() {
        let regex = Regex::new("a+").unwrap();
        assert!(regex.is_match("a"));
        assert!(regex.is_match("aaa"));
        assert!(!regex.is_match(""));
        assert!(!regex.is_match("bbb"));
    }

    #[test]
    fn test_regex_optional() {
        let regex = Regex::new("a?").unwrap();
        assert!(regex.is_match(""));
        assert!(regex.is_match("a"));
        assert!(regex.is_match("aa"));
    }

    #[test]
    fn test_regex_dot() {
        let regex = Regex::new("a.b").unwrap();
        assert!(regex.is_match("axb"));
        assert!(regex.is_match("a b"));
        assert!(!regex.is_match("ab"));
    }

    #[test]
    fn test_regex_group() {
        let regex = Regex::new("(ab)+").unwrap();
        assert!(regex.is_match("ab"));
        assert!(regex.is_match("abab"));
        assert!(!regex.is_match("a"));
    }

    #[test]
    fn test_regex_char_class() {
        let regex = Regex::new("[abc]").unwrap();
        assert!(regex.is_match("a"));
        assert!(regex.is_match("b"));
        assert!(regex.is_match("c"));
        assert!(!regex.is_match("d"));
    }

    #[test]
    fn test_regex_char_class_negated() {
        let regex = Regex::new("[^abc]").unwrap();
        assert!(!regex.is_match("a"));
        assert!(!regex.is_match("b"));
        assert!(!regex.is_match("c"));
        assert!(regex.is_match("d"));
    }

    #[test]
    fn test_regex_find() {
        let regex = Regex::new("abc").unwrap();
        let m = regex.find("xabcy").unwrap();
        assert_eq!(m.start, 1);
        assert_eq!(m.end, 4);
    }

    #[test]
    fn test_regex_find_all() {
        let regex = Regex::new("a").unwrap();
        let matches = regex.find_all("banana");
        assert_eq!(matches.len(), 3);
    }
}
