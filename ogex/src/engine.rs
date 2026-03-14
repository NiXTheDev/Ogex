//! Regex matching engine with backreference support
//!
//! This module provides the actual regex matching functionality,
//! including NFA simulation and backreference handling.

use crate::nfa::{Nfa, StateId, Transition};
use std::collections::HashMap;

/// Dense vector storage for capture groups (index-based for better cache locality)
/// Index 0 is unused (groups are 1-indexed), so groups[n] gives group n's capture
type GroupStorage = Vec<Option<(usize, usize)>>;

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
    /// Captured groups (index-based: groups[n] = Some((start, end)) for group n)
    /// Index 0 is unused (groups are 1-indexed)
    pub groups: GroupStorage,
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
        let idx = n as usize;
        if idx >= self.groups.len() {
            None
        } else {
            self.groups[idx]
        }
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

    /// Try to match the pattern at a specific position without trying other positions
    /// Used for lookahead assertions - checks if pattern matches at current position
    pub fn try_match_at(&self, input: &str, pos: usize) -> bool {
        let mut simulator = NfaSimulator::new(&self.nfa, input, pos);
        simulator.run().is_some()
    }
}

/// A state in the NFA simulation that includes capture group information
#[derive(Debug, Clone)]
struct SimState {
    state_id: StateId,
    /// Capture groups (index-based, index 0 unused)
    groups: GroupStorage,
}

impl SimState {
    /// Create a new state with empty groups (sized for max_groups)
    fn new(state_id: StateId, max_groups: u32) -> Self {
        // +1 because groups are 1-indexed, index 0 is unused
        SimState {
            state_id,
            groups: vec![None; max_groups as usize],
        }
    }

    fn with_groups(state_id: StateId, groups: GroupStorage) -> Self {
        SimState { state_id, groups }
    }
}

/// NFA simulator for pattern matching
#[allow(clippy::type_complexity)]
struct NfaSimulator<'a> {
    nfa: &'a Nfa,
    _input: &'a str,
    /// Input as bytes for ASCII mode, or as chars for Unicode mode
    input_bytes: &'a [u8],
    /// Whether to use ASCII/byte mode (true) or Unicode/char mode (false)
    ascii_mode: bool,
    start_pos: usize,
    /// Memoization cache: (state_id, position) -> Option<groups> (Some if can reach accept, None if cannot)
    memo: HashMap<(StateId, usize), Option<GroupStorage>>,
}

impl<'a> NfaSimulator<'a> {
    fn new(nfa: &'a Nfa, input: &'a str, start_pos: usize) -> Self {
        let ascii_mode = nfa.is_ascii_only();
        NfaSimulator {
            nfa,
            _input: input,
            input_bytes: input.as_bytes(),
            ascii_mode,
            start_pos,
            memo: HashMap::new(),
        }
    }

    /// Check if the result for a given state and position is memoized
    fn check_memoized(&self, state_id: StateId, pos: usize) -> Option<Option<GroupStorage>> {
        self.memo.get(&(state_id, pos)).cloned()
    }

    /// Store the result in the memoization cache
    fn store_memoized(&mut self, state_id: StateId, pos: usize, result: Option<GroupStorage>) {
        self.memo.insert((state_id, pos), result);
    }

    #[allow(clippy::type_complexity)]
    fn run(&mut self) -> Option<Match> {
        // Determine input length and get current character/byte
        let input_len = if self.ascii_mode {
            self.input_bytes.len()
        } else {
            // For Unicode mode, we need char length
            self._input.chars().count()
        };

        let mut pos = self.start_pos;
        let mut last_accept: Option<(usize, GroupStorage)> = None;

        // Initialize with start state
        let mut current_states = vec![SimState::new(self.nfa.start, self.nfa.next_group_id())];

        // Apply epsilon closure to start state
        current_states = self.epsilon_closure(&current_states, pos);

        // Check if start state is accepting (empty match)
        // Use memoization for each state in the closure
        current_states = self.memoize_closure(&current_states, pos, &mut last_accept);

        while pos < input_len {
            let c = if self.ascii_mode {
                // ASCII mode: use bytes
                self.input_bytes[pos] as char
            } else {
                // Unicode mode: use chars
                self._input.chars().nth(pos).unwrap_or('\0')
            };

            let (new_states, chars_consumed) = self.step_with_backrefs(&current_states, c, pos);
            current_states = new_states;

            if current_states.is_empty() {
                break;
            }

            pos += chars_consumed;

            // Apply epsilon closure and use memoization
            current_states = self.epsilon_closure(&current_states, pos);
            current_states = self.memoize_closure(&current_states, pos, &mut last_accept);
        }

        // Try to reach accept state via epsilon transitions (for end anchors)
        current_states = self.epsilon_closure(&current_states, pos);
        // Use memoization for final epsilon closure (result not needed after)
        self.memoize_closure(&current_states, pos, &mut last_accept);

        last_accept.map(|(end, groups)| Match {
            start: self.start_pos,
            end,
            groups,
            named_groups: HashMap::new(),
        })
    }

    /// Apply memoization to a closure of states at a position.
    /// Returns filtered states and updates last_accept if any state leads to accept.
    #[allow(clippy::type_complexity)]
    fn memoize_closure(
        &mut self,
        states: &[SimState],
        pos: usize,
        last_accept: &mut Option<(usize, GroupStorage)>,
    ) -> Vec<SimState> {
        let mut result = Vec::new();

        for sim_state in states {
            // Check memo cache for this state at this position
            match self.check_memoized(sim_state.state_id, pos) {
                // Already computed: if it leads to accept, update last_accept
                Some(Some(groups)) => {
                    *last_accept = Some((pos, groups));
                    // Still add the state for continued exploration
                    result.push(sim_state.clone());
                }
                // Already computed: this state cannot lead to accept, skip it
                Some(None) => {
                    // Skip this state - we've already proven it can't lead to accept
                }
                // Not memoized: need to check if accepting and memoize the result
                None => {
                    let can_accept = self.find_accepting(std::slice::from_ref(sim_state));
                    if let Some(groups) = can_accept.clone() {
                        *last_accept = Some((pos, groups));
                    }
                    // Store result in memo (None if cannot reach accept, Some(groups) if can)
                    self.store_memoized(sim_state.state_id, pos, can_accept);
                    result.push(sim_state.clone());
                }
            }
        }

        result
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
                        let idx = *group_id as usize;
                        if idx < sim_state.groups.len()
                            && let Some((start, end)) = sim_state.groups[idx]
                        {
                            if self.ascii_mode {
                                // ASCII mode: use bytes
                                let captured = &self.input_bytes[start..end];
                                let remaining = &self.input_bytes[pos..];
                                if remaining.starts_with(captured) {
                                    let consumed = captured.len().max(1);
                                    let new_state =
                                        SimState::with_groups(*target, sim_state.groups.clone());
                                    new_states.push(new_state);
                                    chars_consumed = chars_consumed.max(consumed);
                                }
                            } else {
                                // Unicode mode: use chars
                                let captured: String =
                                    self._input.chars().skip(start).take(end - start).collect();
                                let remaining: String = self._input.chars().skip(pos).collect();
                                if remaining.starts_with(&captured) {
                                    let consumed = captured.len().max(1);
                                    let new_state =
                                        SimState::with_groups(*target, sim_state.groups.clone());
                                    new_states.push(new_state);
                                    chars_consumed = chars_consumed.max(consumed);
                                }
                            }
                        }
                    }
                    Transition::BackrefRelative(relative) => {
                        // Resolve relative backreference (\g{-n})
                        if let Some(group_id) = self.nfa.resolve_relative(*relative) {
                            let idx = group_id as usize;
                            if idx < sim_state.groups.len()
                                && let Some((start, end)) = sim_state.groups[idx]
                            {
                                if self.ascii_mode {
                                    // ASCII mode: use bytes
                                    let captured = &self.input_bytes[start..end];
                                    let remaining = &self.input_bytes[pos..];
                                    if remaining.starts_with(captured) {
                                        let consumed = captured.len().max(1);
                                        let new_state = SimState::with_groups(
                                            *target,
                                            sim_state.groups.clone(),
                                        );
                                        new_states.push(new_state);
                                        chars_consumed = chars_consumed.max(consumed);
                                    }
                                } else {
                                    // Unicode mode: use chars
                                    let captured: String =
                                        self._input.chars().skip(start).take(end - start).collect();
                                    let remaining: String = self._input.chars().skip(pos).collect();
                                    if remaining.starts_with(&captured) {
                                        let consumed = captured.len().max(1);
                                        let new_state = SimState::with_groups(
                                            *target,
                                            sim_state.groups.clone(),
                                        );
                                        new_states.push(new_state);
                                        chars_consumed = chars_consumed.max(consumed);
                                    }
                                }
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
                    tc.eq_ignore_ascii_case(&c)
                } else {
                    *tc == c
                }
            }
            Transition::CharClass {
                negated: _negated,
                lookup,
            } => {
                // O(1) lookup using pre-computed table (negation already handled in lookup)
                if c as u32 > 255 {
                    // For non-ASCII, fall back to simple check
                    false
                } else {
                    let byte_idx = (c as u8 / 8) as usize;
                    let bit_idx = c as u8 % 8;
                    (lookup[byte_idx] & (1 << bit_idx)) != 0
                }
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

            // Use pre-computed epsilon closure for pure epsilon transitions
            let epsilon_targets = self.nfa.get_epsilon_closure(sim_state.state_id);
            for &target in epsilon_targets {
                if !closure.iter().any(|s| s.state_id == target) {
                    stack.push(SimState::with_groups(target, sim_state.groups.clone()));
                }
            }

            // Handle position-dependent transitions (anchors, word boundaries, groups)
            for (transition, target) in &self.nfa.states[sim_state.state_id].transitions {
                // Skip pure epsilon - already handled by pre-computed closure
                if matches!(transition, Transition::Epsilon) {
                    continue;
                }

                if closure.iter().any(|s| s.state_id == *target) {
                    continue;
                }

                match transition {
                    Transition::StartAnchor => {
                        if self.nfa.mode_flags.multiline {
                            // In multiline mode, ^ matches at start of string or after newline
                            let is_start = pos == self.start_pos;
                            let is_after_newline = if self.ascii_mode {
                                pos > 0 && self.input_bytes[pos - 1] == b'\n'
                            } else {
                                pos > 0 && self._input.chars().nth(pos - 1) == Some('\n')
                            };
                            if is_start || is_after_newline {
                                stack
                                    .push(SimState::with_groups(*target, sim_state.groups.clone()));
                            }
                        } else if self.start_pos == 0 && pos == self.start_pos {
                            stack.push(SimState::with_groups(*target, sim_state.groups.clone()));
                        }
                    }
                    Transition::EndAnchor => {
                        if self.nfa.mode_flags.multiline {
                            // In multiline mode, $ matches at end of string or before newline
                            let input_len = if self.ascii_mode {
                                self.input_bytes.len()
                            } else {
                                self._input.chars().count()
                            };
                            let is_end = pos == input_len;
                            let is_before_newline = if self.ascii_mode {
                                self.input_bytes.get(pos) == Some(&b'\n')
                            } else {
                                self._input.chars().nth(pos) == Some('\n')
                            };
                            if is_end || is_before_newline {
                                stack
                                    .push(SimState::with_groups(*target, sim_state.groups.clone()));
                            }
                        } else {
                            let input_len = if self.ascii_mode {
                                self.input_bytes.len()
                            } else {
                                self._input.chars().count()
                            };
                            if pos == input_len {
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
                        let idx = *group_id as usize;
                        if idx < new_groups.len() {
                            new_groups[idx] = Some((pos, pos)); // Start capturing
                        }
                        stack.push(SimState::with_groups(*target, new_groups));
                    }
                    Transition::GroupEnd(group_id) => {
                        let mut new_groups = sim_state.groups.clone();
                        let idx = *group_id as usize;
                        if idx < new_groups.len()
                            && let Some((start, _)) = new_groups[idx]
                        {
                            new_groups[idx] = Some((start, pos)); // End capturing
                        }
                        stack.push(SimState::with_groups(*target, new_groups));
                    }
                    Transition::Lookahead(inner_nfa) => {
                        // Check if the inner pattern matches at the current position
                        // without consuming input (lookahead is zero-width)
                        if self.check_lookahead(inner_nfa, pos) {
                            stack.push(SimState::with_groups(*target, sim_state.groups.clone()));
                        }
                    }
                    Transition::NegativeLookahead(inner_nfa) => {
                        // Check if the inner pattern does NOT match at the current position
                        if !self.check_lookahead(inner_nfa, pos) {
                            stack.push(SimState::with_groups(*target, sim_state.groups.clone()));
                        }
                    }
                    _ => {} // Char/CharClass handled in step
                }
            }
        }

        closure
    }

    fn find_accepting(&self, states: &[SimState]) -> Option<GroupStorage> {
        states
            .iter()
            .find(|s| s.state_id == self.nfa.accept)
            .map(|s| s.groups.clone())
    }

    fn is_word_boundary(&self, pos: usize) -> bool {
        let (left_is_word, right_is_word) = if self.ascii_mode {
            // ASCII mode: use bytes
            let left_is_word = pos > 0 && self.is_word_byte(self.input_bytes[pos - 1]);
            let right_is_word =
                pos < self.input_bytes.len() && self.is_word_byte(self.input_bytes[pos]);
            (left_is_word, right_is_word)
        } else {
            // Unicode mode: use chars
            let left_is_word =
                pos > 0 && self.is_word_char(self._input.chars().nth(pos - 1).unwrap_or('\0'));
            let right_is_word = pos < self._input.chars().count()
                && self.is_word_char(self._input.chars().nth(pos).unwrap_or('\0'));
            (left_is_word, right_is_word)
        };
        left_is_word != right_is_word
    }

    fn is_word_byte(&self, b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'_'
    }

    fn is_word_char(&self, c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    /// Check if an inner NFA matches at a specific position without consuming input
    /// Used for lookahead assertions
    fn check_lookahead(&self, inner_nfa: &Nfa, pos: usize) -> bool {
        // Create a regex from the inner NFA and try to match at the position
        let regex = Regex {
            nfa: inner_nfa.clone(),
        };
        // Use try_match_at to check if pattern matches without consuming beyond
        regex.try_match_at(self._input, pos)
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
