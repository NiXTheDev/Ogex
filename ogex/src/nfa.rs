//! NFA (Nondeterministic Finite Automaton) construction and simulation
//!
//! This module implements Thompson's construction algorithm to build an NFA
//! from a regex AST, and provides NFA simulation for pattern matching.

use crate::ast::{ClassItem, Expr, Quantifier};
use std::collections::{HashMap, HashSet};

/// An NFA state ID
pub type StateId = usize;

/// A transition in the NFA
#[derive(Debug, Clone, PartialEq)]
pub enum Transition {
    /// Transition on a specific character
    Char(char),
    /// Transition on any character (dot)
    Any,
    /// Epsilon transition (no input consumed)
    Epsilon,
    /// Transition matching a character class
    CharClass {
        negated: bool,
        items: Vec<ClassItem>,
    },
    /// Start of a capture group
    GroupStart(u32),
    /// End of a capture group
    GroupEnd(u32),
    /// Backreference transition (absolute group index)
    Backref(u32),
    /// Relative backreference transition (\g{-n})
    /// Resolved at match time against numbered groups only
    BackrefRelative(i32),
    /// Start of string anchor
    StartAnchor,
    /// End of string anchor
    EndAnchor,
    /// Word boundary assertion
    WordBoundary,
    /// Non-word boundary assertion
    NonWordBoundary,
}

/// An NFA state
#[derive(Debug, Clone)]
pub struct State {
    /// Transitions from this state
    pub transitions: Vec<(Transition, StateId)>,
    /// Whether this is an accepting state
    pub is_accepting: bool,
}

impl State {
    fn new() -> Self {
        State {
            transitions: Vec::new(),
            is_accepting: false,
        }
    }
}

/// An NFA (Nondeterministic Finite Automaton)
#[derive(Debug)]
pub struct Nfa {
    /// All states in the NFA
    pub states: Vec<State>,
    /// The start state
    pub start: StateId,
    /// The accepting state
    pub accept: StateId,
    /// Next state ID to allocate
    next_state_id: StateId,
    /// Next group ID to allocate
    next_group_id: u32,
    /// Named group mapping (name -> group_id)
    named_groups: HashMap<String, u32>,
    /// List of numbered (non-named) group indices, in order of appearance
    /// Used for relative backreference resolution
    numbered_groups: Vec<u32>,
}

impl Nfa {
    /// Create a new empty NFA
    pub fn new() -> Self {
        Nfa {
            states: Vec::new(),
            start: 0,
            accept: 0,
            next_state_id: 0,
            next_group_id: 1, // Group 0 is the entire match
            named_groups: HashMap::new(),
            numbered_groups: Vec::new(),
        }
    }

    /// Allocate a new state and return its ID
    fn new_state(&mut self) -> StateId {
        let id = self.next_state_id;
        self.next_state_id += 1;
        self.states.push(State::new());
        id
    }

    /// Add a transition between states
    fn add_transition(&mut self, from: StateId, transition: Transition, to: StateId) {
        self.states[from].transitions.push((transition, to));
    }

    /// Build an NFA from an AST expression
    pub fn from_expr(expr: &Expr) -> Self {
        let mut nfa = Nfa::new();
        let (start, accept) = nfa.compile_expr(expr);
        nfa.start = start;
        nfa.accept = accept;
        nfa.states[accept].is_accepting = true;
        nfa
    }

    /// Compile an expression and return (start, accept) state IDs
    fn compile_expr(&mut self, expr: &Expr) -> (StateId, StateId) {
        match expr {
            Expr::Empty => self.compile_empty(),
            Expr::Literal(c) => self.compile_char(*c),
            Expr::Any => self.compile_any(),
            Expr::Sequence(exprs) => self.compile_sequence(exprs),
            Expr::Alternation(exprs) => self.compile_alternation(exprs),
            Expr::CharacterClass(cc) => self.compile_char_class(cc.negated, &cc.items),
            Expr::Quantified { expr, quantifier } => self.compile_quantified(expr, *quantifier),
            Expr::Group(expr) => self.compile_group(expr, None),
            Expr::NonCapturingGroup(expr) => self.compile_expr(expr),
            Expr::NamedGroup { name, pattern } => self.compile_group(pattern, Some(name.clone())),
            Expr::AtomicGroup(expr) => self.compile_expr(expr),
            Expr::StartAnchor => self.compile_start_anchor(),
            Expr::EndAnchor => self.compile_end_anchor(),
            Expr::Backreference(n) => self.compile_backref(*n),
            Expr::RelativeBackreference(n) => self.compile_backref_relative(*n),
            Expr::NamedBackreference(name) => {
                // Resolve the named backreference to a group ID
                if let Some(&group_id) = self.named_groups.get(name) {
                    self.compile_backref(group_id)
                } else {
                    // Unknown group name - compile as backref 0 (will never match)
                    self.compile_backref(0)
                }
            }
            Expr::Shorthand(c) => self.compile_shorthand(*c),
            Expr::WordBoundary => self.compile_word_boundary(false),
            Expr::NonWordBoundary => self.compile_word_boundary(true),

            // Handle new assertion and group types
            _ => self.compile_empty(),
        }
    }

    /// Compile an empty expression
    fn compile_empty(&mut self) -> (StateId, StateId) {
        let start = self.new_state();
        let accept = self.new_state();
        self.add_transition(start, Transition::Epsilon, accept);
        (start, accept)
    }

    /// Compile a literal character
    fn compile_char(&mut self, c: char) -> (StateId, StateId) {
        let start = self.new_state();
        let accept = self.new_state();
        self.add_transition(start, Transition::Char(c), accept);
        (start, accept)
    }

    /// Compile 'any' (.)
    fn compile_any(&mut self) -> (StateId, StateId) {
        let start = self.new_state();
        let accept = self.new_state();
        self.add_transition(start, Transition::Any, accept);
        (start, accept)
    }

    /// Compile a sequence
    fn compile_sequence(&mut self, exprs: &[Expr]) -> (StateId, StateId) {
        if exprs.is_empty() {
            return self.compile_empty();
        }

        let mut start = None;
        let mut prev_accept = None;

        for expr in exprs {
            let (s, a) = self.compile_expr(expr);
            if start.is_none() {
                start = Some(s);
            }
            if let Some(prev) = prev_accept {
                self.add_transition(prev, Transition::Epsilon, s);
            }
            prev_accept = Some(a);
        }

        (start.unwrap(), prev_accept.unwrap())
    }

    /// Compile alternation (a|b|c)
    fn compile_alternation(&mut self, exprs: &[Expr]) -> (StateId, StateId) {
        if exprs.is_empty() {
            return self.compile_empty();
        }
        if exprs.len() == 1 {
            return self.compile_expr(&exprs[0]);
        }

        let start = self.new_state();
        let accept = self.new_state();

        for expr in exprs {
            let (s, a) = self.compile_expr(expr);
            self.add_transition(start, Transition::Epsilon, s);
            self.add_transition(a, Transition::Epsilon, accept);
        }

        (start, accept)
    }

    /// Compile a character class
    fn compile_char_class(&mut self, negated: bool, items: &[ClassItem]) -> (StateId, StateId) {
        let start = self.new_state();
        let accept = self.new_state();
        self.add_transition(
            start,
            Transition::CharClass {
                negated,
                items: items.to_vec(),
            },
            accept,
        );
        (start, accept)
    }

    /// Compile a quantified expression
    fn compile_quantified(&mut self, expr: &Expr, quantifier: Quantifier) -> (StateId, StateId) {
        let (inner_start, inner_accept) = self.compile_expr(expr);
        let start = self.new_state();
        let accept = self.new_state();

        match quantifier {
            Quantifier::ZeroOrMore => {
                // *: Greedy - prefer to match more
                self.add_transition(start, Transition::Epsilon, inner_start);
                self.add_transition(start, Transition::Epsilon, accept);
                self.add_transition(inner_accept, Transition::Epsilon, inner_start);
                self.add_transition(inner_accept, Transition::Epsilon, accept);
            }
            Quantifier::ZeroOrMoreLazy => {
                // *?: Lazy - prefer to match less (exit first)
                self.add_transition(start, Transition::Epsilon, accept);
                self.add_transition(start, Transition::Epsilon, inner_start);
                self.add_transition(inner_accept, Transition::Epsilon, accept);
                self.add_transition(inner_accept, Transition::Epsilon, inner_start);
            }
            Quantifier::OneOrMore => {
                // +: Greedy - prefer to match more
                self.add_transition(start, Transition::Epsilon, inner_start);
                self.add_transition(inner_accept, Transition::Epsilon, inner_start);
                self.add_transition(inner_accept, Transition::Epsilon, accept);
            }
            Quantifier::OneOrMoreLazy => {
                // +?: Lazy - prefer to match less
                self.add_transition(start, Transition::Epsilon, inner_start);
                self.add_transition(inner_accept, Transition::Epsilon, accept);
                self.add_transition(inner_accept, Transition::Epsilon, inner_start);
            }
            Quantifier::Optional => {
                // ?: Can skip or match once
                self.add_transition(start, Transition::Epsilon, inner_start);
                self.add_transition(start, Transition::Epsilon, accept);
                self.add_transition(inner_accept, Transition::Epsilon, accept);
            }
            Quantifier::Exactly(n) => {
                // {n}: Match exactly n times (no lazy/greedy distinction)
                return self.compile_repeat_exact(expr, n);
            }
            Quantifier::AtLeast(n) => {
                // {n,}: Greedy - prefer to match more
                return self.compile_repeat_at_least(expr, n);
            }
            Quantifier::AtLeastLazy(n) => {
                // {n,}?: Lazy - prefer to match less
                return self.compile_repeat_at_least_lazy(expr, n);
            }
            Quantifier::Between(n, m) => {
                // {n,m}: Greedy - prefer to match more
                return self.compile_repeat_between(expr, n, m);
            }
            Quantifier::BetweenLazy(n, m) => {
                // {n,m}?: Lazy - prefer to match less
                return self.compile_repeat_between_lazy(expr, n, m);
            }
        }

        (start, accept)
    }

    /// Compile repeat exactly n times
    fn compile_repeat_exact(&mut self, expr: &Expr, n: u32) -> (StateId, StateId) {
        if n == 0 {
            return self.compile_empty();
        }
        if n == 1 {
            return self.compile_expr(expr);
        }

        let mut start = None;
        let mut prev_accept = None;

        for _ in 0..n {
            let (s, a) = self.compile_expr(expr);
            if start.is_none() {
                start = Some(s);
            }
            if let Some(prev) = prev_accept {
                self.add_transition(prev, Transition::Epsilon, s);
            }
            prev_accept = Some(a);
        }

        (start.unwrap(), prev_accept.unwrap())
    }

    /// Compile repeat at least n times
    fn compile_repeat_at_least(&mut self, expr: &Expr, n: u32) -> (StateId, StateId) {
        // Match exactly n times, then add a * (zero or more)
        let (exact_start, exact_accept) = self.compile_repeat_exact(expr, n);
        let (star_start, star_accept) = self.compile_quantified(expr, Quantifier::ZeroOrMore);

        self.add_transition(exact_accept, Transition::Epsilon, star_start);
        (exact_start, star_accept)
    }

    /// Compile repeat at least n times (lazy)
    fn compile_repeat_at_least_lazy(&mut self, expr: &Expr, n: u32) -> (StateId, StateId) {
        // Match exactly n times, then add a *? (zero or more lazy)
        let (exact_start, exact_accept) = self.compile_repeat_exact(expr, n);
        let (star_start, star_accept) = self.compile_quantified(expr, Quantifier::ZeroOrMoreLazy);

        self.add_transition(exact_accept, Transition::Epsilon, star_start);
        (exact_start, star_accept)
    }

    /// Compile repeat between n and m times
    fn compile_repeat_between(&mut self, expr: &Expr, n: u32, m: u32) -> (StateId, StateId) {
        if n == m {
            return self.compile_repeat_exact(expr, n);
        }

        // Match exactly n times, then match (m-n) optional times
        let (exact_start, exact_accept) = self.compile_repeat_exact(expr, n);

        let start = self.new_state();
        let accept = self.new_state();

        self.add_transition(start, Transition::Epsilon, exact_start);

        // Add (m-n) optional matches
        let mut prev_accept = exact_accept;
        for _ in 0..(m - n) {
            let (s, a) = self.compile_expr(expr);
            self.add_transition(prev_accept, Transition::Epsilon, s);
            self.add_transition(prev_accept, Transition::Epsilon, accept);
            prev_accept = a;
        }

        self.add_transition(prev_accept, Transition::Epsilon, accept);
        (start, accept)
    }

    /// Compile repeat between n and m times (lazy)
    fn compile_repeat_between_lazy(&mut self, expr: &Expr, n: u32, m: u32) -> (StateId, StateId) {
        if n == m {
            return self.compile_repeat_exact(expr, n);
        }

        // Match exactly n times, then match (m-n) optional times (lazy)
        let (exact_start, exact_accept) = self.compile_repeat_exact(expr, n);

        let start = self.new_state();
        let accept = self.new_state();

        self.add_transition(start, Transition::Epsilon, exact_start);

        // Add (m-n) optional matches (lazy order)
        let mut prev_accept = exact_accept;
        for _ in 0..(m - n) {
            let (s, a) = self.compile_expr(expr);
            // Lazy: try to exit first, then match
            self.add_transition(prev_accept, Transition::Epsilon, accept);
            self.add_transition(prev_accept, Transition::Epsilon, s);
            prev_accept = a;
        }

        self.add_transition(prev_accept, Transition::Epsilon, accept);
        (start, accept)
    }

    /// Compile a group (capturing or named)
    fn compile_group(&mut self, expr: &Expr, name: Option<String>) -> (StateId, StateId) {
        let group_id = self.next_group_id;
        self.next_group_id += 1;

        // Register named group if applicable, otherwise track as numbered
        if let Some(n) = name {
            self.named_groups.insert(n, group_id);
        } else {
            self.numbered_groups.push(group_id);
        }

        let start = self.new_state();
        let (inner_start, inner_accept) = self.compile_expr(expr);
        let accept = self.new_state();

        self.add_transition(start, Transition::GroupStart(group_id), inner_start);
        self.add_transition(inner_accept, Transition::GroupEnd(group_id), accept);

        (start, accept)
    }

    /// Compile start anchor (^)
    fn compile_start_anchor(&mut self) -> (StateId, StateId) {
        let start = self.new_state();
        let accept = self.new_state();
        self.add_transition(start, Transition::StartAnchor, accept);
        (start, accept)
    }

    /// Compile end anchor ($)
    fn compile_end_anchor(&mut self) -> (StateId, StateId) {
        let start = self.new_state();
        let accept = self.new_state();
        self.add_transition(start, Transition::EndAnchor, accept);
        (start, accept)
    }

    /// Compile backreference
    fn compile_backref(&mut self, n: u32) -> (StateId, StateId) {
        let start = self.new_state();
        let accept = self.new_state();
        self.add_transition(start, Transition::Backref(n), accept);
        (start, accept)
    }

    /// Compile relative backreference (\g{-n})
    /// Resolution happens at match time against numbered groups only
    fn compile_backref_relative(&mut self, n: i32) -> (StateId, StateId) {
        let start = self.new_state();
        let accept = self.new_state();
        self.add_transition(start, Transition::BackrefRelative(n), accept);
        (start, accept)
    }

    /// Compile shorthand character class (\w, \d, \s, etc.)
    fn compile_shorthand(&mut self, c: char) -> (StateId, StateId) {
        let start = self.new_state();
        let accept = self.new_state();
        self.add_transition(
            start,
            Transition::CharClass {
                negated: c.is_ascii_uppercase(), // \W, \D, \S are negated
                items: vec![crate::ast::ClassItem::Shorthand(c.to_ascii_lowercase())],
            },
            accept,
        );
        (start, accept)
    }

    /// Compile word boundary assertion (\b or \B)
    fn compile_word_boundary(&mut self, negated: bool) -> (StateId, StateId) {
        let start = self.new_state();
        let accept = self.new_state();
        if negated {
            self.add_transition(start, Transition::NonWordBoundary, accept);
        } else {
            self.add_transition(start, Transition::WordBoundary, accept);
        }
        (start, accept)
    }

    /// Compute epsilon closure of a set of states
    pub fn epsilon_closure(&self, states: &HashSet<StateId>) -> HashSet<StateId> {
        let mut closure = states.clone();
        let mut stack: Vec<_> = states.iter().copied().collect();

        while let Some(state) = stack.pop() {
            for (transition, target) in &self.states[state].transitions {
                if matches!(transition, Transition::Epsilon) && !closure.contains(target) {
                    closure.insert(*target);
                    stack.push(*target);
                }
            }
        }

        closure
    }

    /// Get the list of numbered (non-named) group indices
    /// Used for relative backreference resolution
    pub fn numbered_groups(&self) -> &[u32] {
        &self.numbered_groups
    }

    /// Get the count of numbered (non-named) groups
    pub fn numbered_group_count(&self) -> usize {
        self.numbered_groups.len()
    }

    /// Resolve a relative backreference (\g{-n}) to an absolute group index
    ///
    /// # Arguments
    /// * `relative` - The negative index (-1 = last numbered group, -2 = second-to-last, etc.)
    ///
    /// # Returns
    /// The absolute group index, or None if invalid
    pub fn resolve_relative(&self, relative: i32) -> Option<u32> {
        if relative >= 0 {
            return None;
        }
        let reverse_index = (-relative) as usize;
        if reverse_index == 0 || reverse_index > self.numbered_groups.len() {
            return None;
        }
        let actual_index = self.numbered_groups.len() - reverse_index;
        Some(self.numbered_groups[actual_index])
    }
}

impl Default for Nfa {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;

    #[test]
    fn test_nfa_from_literal() {
        let expr = Expr::literal('a');
        let nfa = Nfa::from_expr(&expr);
        assert_eq!(nfa.states.len(), 2);
        assert!(!nfa.states[nfa.start].is_accepting);
        assert!(nfa.states[nfa.accept].is_accepting);
    }

    #[test]
    fn test_nfa_from_sequence() {
        let expr = Expr::sequence(vec![Expr::literal('a'), Expr::literal('b')]);
        let nfa = Nfa::from_expr(&expr);
        assert!(nfa.states.len() >= 4); // At least 2 chars + transitions
    }

    #[test]
    fn test_nfa_from_alternation() {
        let expr = Expr::alternation(vec![Expr::literal('a'), Expr::literal('b')]);
        let nfa = Nfa::from_expr(&expr);
        assert!(nfa.states.len() >= 6); // Branch states + 2 chars
    }

    #[test]
    fn test_nfa_from_star() {
        let expr = Expr::quantified(Expr::literal('a'), Quantifier::ZeroOrMore);
        let nfa = Nfa::from_expr(&expr);
        assert!(nfa.states.len() >= 4); // Star requires branch states
    }

    #[test]
    fn test_nfa_from_plus() {
        let expr = Expr::quantified(Expr::literal('a'), Quantifier::OneOrMore);
        let nfa = Nfa::from_expr(&expr);
        assert!(nfa.states.len() >= 4);
    }

    #[test]
    fn test_nfa_from_optional() {
        let expr = Expr::quantified(Expr::literal('a'), Quantifier::Optional);
        let nfa = Nfa::from_expr(&expr);
        assert!(nfa.states.len() >= 4);
    }

    #[test]
    fn test_epsilon_closure() {
        let mut nfa = Nfa::new();
        let s0 = nfa.new_state();
        let s1 = nfa.new_state();
        let s2 = nfa.new_state();

        nfa.add_transition(s0, Transition::Epsilon, s1);
        nfa.add_transition(s1, Transition::Epsilon, s2);

        let closure = nfa.epsilon_closure(&[s0].into_iter().collect());
        assert!(closure.contains(&s0));
        assert!(closure.contains(&s1));
        assert!(closure.contains(&s2));
    }

    #[test]
    fn test_nfa_from_group() {
        let expr = Expr::group(Expr::literal('a'));
        let nfa = Nfa::from_expr(&expr);
        assert!(nfa.states.len() >= 2);
    }

    #[test]
    fn test_nfa_from_named_group() {
        let expr = Expr::named_group("test", Expr::literal('a'));
        let nfa = Nfa::from_expr(&expr);
        assert!(nfa.states.len() >= 2);
    }

    #[test]
    fn test_nfa_from_complex() {
        // (a|b)*c
        let expr = Expr::sequence(vec![
            Expr::quantified(
                Expr::alternation(vec![Expr::literal('a'), Expr::literal('b')]),
                Quantifier::ZeroOrMore,
            ),
            Expr::literal('c'),
        ]);
        let nfa = Nfa::from_expr(&expr);
        assert!(nfa.states.len() >= 8);
    }

    #[test]
    fn test_nfa_from_atomic_group() {
        let expr = Expr::AtomicGroup(Box::new(Expr::literal('a')));
        let nfa = Nfa::from_expr(&expr);
        assert!(nfa.states.len() >= 2);
    }
}
