//! Regex matching engine with backreference support
//!
//! This module provides the actual regex matching functionality,
//! including NFA simulation and backreference handling.

use crate::ast::ClassItem;
use crate::nfa::{Nfa, StateId, Transition};
use std::collections::HashMap;

/// Mode flags for regex matching
#[derive(Debug, Clone, Default)]
pub struct ModeFlags {
    /// Case insensitive matching (@i)
    pub case_insensitive: bool,
    /// Multiline mode - ^ and $ match line boundaries (@m)
    pub multiline: bool,
    /// Dot matches newline (@s)
    pub dotall: bool,
    /// Extended mode - ignore whitespace, allow # comments (@x)
    pub extended: bool,
}

impl ModeFlags {
    /// Parse mode flags from a string like "imsx"
    pub fn from_string(flags: &str) -> Self {
        let mut mode = ModeFlags::default();
        for c in flags.chars() {
            match c {
                'i' => mode.case_insensitive = true,
                'm' => mode.multiline = true,
                's' => mode.dotall = true,
                'x' => mode.extended = true,
                _ => {}
            }
        }
        mode
    }

    /// Merge with another set of flags (for nested mode flags groups)
    pub fn merge(&mut self, other: &ModeFlags) {
        self.case_insensitive = self.case_insensitive || other.case_insensitive;
        self.multiline = self.multiline || other.multiline;
        self.dotall = self.dotall || other.dotall;
        self.extended = self.extended || other.extended;
    }
}

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

/// A state in the NFA simulation that includes capture group information
#[derive(Debug, Clone)]
struct SimState {
    state_id: StateId,
    groups: HashMap<u32, (usize, usize)>,
}

impl SimState {
    fn new(state_id: StateId) -> Self {
        SimState {
            state_id,
            groups: HashMap::new(),
        }
    }

    fn with_groups(state_id: StateId, groups: HashMap<u32, (usize, usize)>) -> Self {
        SimState { state_id, groups }
    }
}

/// NFA simulator for pattern matching
struct NfaSimulator<'a> {
    nfa: &'a Nfa,
    _input: &'a str,
    input_chars: Vec<char>,
    start_pos: usize,
}

impl<'a> NfaSimulator<'a> {
    fn new(nfa: &'a Nfa, input: &'a str, start_pos: usize) -> Self {
        NfaSimulator {
            nfa,
            _input: input,
            input_chars: input.chars().collect(),
            start_pos,
        }
    }

    fn run(&mut self) -> Option<Match> {
        let mut pos = self.start_pos;
        let mut last_accept: Option<(usize, HashMap<u32, (usize, usize)>)> = None;

        // Initialize with start state
        let mut current_states = vec![SimState::new(self.nfa.start)];

        // Apply epsilon closure to start state
        current_states = self.epsilon_closure(&current_states, pos);

        // Check if start state is accepting (empty match)
        if let Some(groups) = self.find_accepting(&current_states) {
            last_accept = Some((pos, groups));
        }

        while pos < self.input_chars.len() {
            let c = self.input_chars[pos];
            let (new_states, chars_consumed) = self.step_with_backrefs(&current_states, c, pos);
            current_states = new_states;

            if current_states.is_empty() {
                break;
            }

            pos += chars_consumed;

            if let Some(groups) = self.find_accepting(&current_states) {
                last_accept = Some((pos, groups));
            }
        }

        // Try to reach accept state via epsilon transitions (for end anchors)
        current_states = self.epsilon_closure(&current_states, pos);
        if let Some(groups) = self.find_accepting(&current_states) {
            last_accept = Some((pos, groups));
        }

        last_accept.map(|(end, groups)| Match {
            start: self.start_pos,
            end,
            groups,
            named_groups: HashMap::new(),
        })
    }

    fn step_with_backrefs(
        &self,
        states: &[SimState],
        c: char,
        pos: usize,
    ) -> (Vec<SimState>, usize) {
        let mut new_states: Vec<SimState> = Vec::new();
        let mut chars_consumed = 1; // Default: consume 1 character

        for sim_state in states {
            for (transition, target) in &self.nfa.states[sim_state.state_id].transitions {
                match transition {
                    Transition::Backref(group_id) => {
                        // Try to match the backreference
                        if let Some((start, end)) = sim_state.groups.get(group_id) {
                            let captured: String = self.input_chars[*start..*end].iter().collect();
                            let remaining: String = self.input_chars[pos..].iter().collect();

                            if remaining.starts_with(&captured) {
                                let consumed = captured.len().max(1);
                                let new_state =
                                    SimState::with_groups(*target, sim_state.groups.clone());
                                new_states.push(new_state);
                                chars_consumed = chars_consumed.max(consumed);
                            }
                        }
                    }
                    Transition::BackrefRelative(relative) => {
                        // Resolve relative backreference (\g{-n})
                        if let Some(group_id) = self.nfa.resolve_relative(*relative)
                            && let Some((start, end)) = sim_state.groups.get(&group_id)
                        {
                            let captured: String = self.input_chars[*start..*end].iter().collect();
                            let remaining: String = self.input_chars[pos..].iter().collect();

                            if remaining.starts_with(&captured) {
                                let consumed = captured.len().max(1);
                                let new_state =
                                    SimState::with_groups(*target, sim_state.groups.clone());
                                new_states.push(new_state);
                                chars_consumed = chars_consumed.max(consumed);
                            }
                        }
                    }
                    _ => {
                        // Regular character transition
                        if let Some(new_state) =
                            self.try_transition(sim_state, transition, *target, c, pos)
                        {
                            new_states.push(new_state);
                        }
                    }
                }
            }
        }

        (
            self.epsilon_closure(&new_states, pos + chars_consumed),
            chars_consumed,
        )
    }

    fn try_transition(
        &self,
        sim_state: &SimState,
        transition: &Transition,
        target: StateId,
        c: char,
        _pos: usize,
    ) -> Option<SimState> {
        let matches = match transition {
            Transition::Char(tc) => {
                if self.nfa.mode_flags.case_insensitive {
                    tc.to_ascii_lowercase() == c.to_ascii_lowercase()
                } else {
                    *tc == c
                }
            }
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
                if *negated { !matched } else { matched }
            }
            Transition::Any => {
                // In dotall mode, . matches any character including newline
                if self.nfa.mode_flags.dotall {
                    true
                } else {
                    c != '\n'
                }
            }
            _ => false, // Epsilon transitions handled in epsilon_closure
        };

        if matches {
            let new_sim_state = SimState::with_groups(target, sim_state.groups.clone());
            Some(new_sim_state)
        } else {
            None
        }
    }

    fn epsilon_closure(&self, states: &[SimState], pos: usize) -> Vec<SimState> {
        // Use a Vec to preserve order (important for lazy vs greedy)
        let mut closure: Vec<SimState> = Vec::new();
        let mut stack: Vec<SimState> = states.to_vec();

        while let Some(sim_state) = stack.pop() {
            // Check if we already have this state
            if closure.iter().any(|s| s.state_id == sim_state.state_id) {
                continue;
            }

            closure.push(sim_state.clone());

            for (transition, target) in &self.nfa.states[sim_state.state_id].transitions {
                if closure.iter().any(|s| s.state_id == *target) {
                    continue;
                }

                match transition {
                    Transition::Epsilon => {
                        stack.push(SimState::with_groups(*target, sim_state.groups.clone()));
                    }
                    Transition::StartAnchor => {
                        if self.nfa.mode_flags.multiline {
                            // In multiline mode, ^ matches at start of string or after newline
                            let is_start = pos == self.start_pos;
                            let is_after_newline =
                                pos > 0 && self.input_chars.get(pos - 1) == Some(&'\n');
                            if is_start || is_after_newline {
                                stack
                                    .push(SimState::with_groups(*target, sim_state.groups.clone()));
                            }
                        } else {
                            if self.start_pos == 0 && pos == self.start_pos {
                                stack
                                    .push(SimState::with_groups(*target, sim_state.groups.clone()));
                            }
                        }
                    }
                    Transition::EndAnchor => {
                        if self.nfa.mode_flags.multiline {
                            // In multiline mode, $ matches at end of string or before newline
                            let is_end = pos == self.input_chars.len();
                            let is_before_newline = self.input_chars.get(pos) == Some(&'\n');
                            if is_end || is_before_newline {
                                stack
                                    .push(SimState::with_groups(*target, sim_state.groups.clone()));
                            }
                        } else {
                            if pos == self.input_chars.len() {
                                stack
                                    .push(SimState::with_groups(*target, sim_state.groups.clone()));
                            }
                        }
                    }
                    Transition::WordBoundary => {
                        if self.is_word_boundary(pos) {
                            stack.push(SimState::with_groups(*target, sim_state.groups.clone()));
                        }
                    }
                    Transition::NonWordBoundary => {
                        if !self.is_word_boundary(pos) {
                            stack.push(SimState::with_groups(*target, sim_state.groups.clone()));
                        }
                    }
                    Transition::GroupStart(group_id) => {
                        let mut new_groups = sim_state.groups.clone();
                        new_groups.insert(*group_id, (pos, pos)); // Start capturing
                        stack.push(SimState::with_groups(*target, new_groups));
                    }
                    Transition::GroupEnd(group_id) => {
                        let mut new_groups = sim_state.groups.clone();
                        if let Some((start, _)) = new_groups.get(group_id) {
                            new_groups.insert(*group_id, (*start, pos)); // End capturing
                        }
                        stack.push(SimState::with_groups(*target, new_groups));
                    }
                    _ => {} // Char/CharClass handled in step
                }
            }
        }

        closure
    }

    fn find_accepting(&self, states: &[SimState]) -> Option<HashMap<u32, (usize, usize)>> {
        states
            .iter()
            .find(|s| s.state_id == self.nfa.accept)
            .map(|s| s.groups.clone())
    }

    fn is_word_boundary(&self, pos: usize) -> bool {
        let left_is_word = pos > 0 && self.is_word_char(self.input_chars[pos - 1]);
        let right_is_word =
            pos < self.input_chars.len() && self.is_word_char(self.input_chars[pos]);
        left_is_word != right_is_word
    }

    fn is_word_char(&self, c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
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
