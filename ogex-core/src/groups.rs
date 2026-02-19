//! Group registry for tracking capture groups
//!
//! This module provides a registry that tracks all capture groups in a regex pattern,
//! mapping names to indices and vice versa. This is essential for:
//! - Resolving backreferences by name
//! - Ensuring group names are unique
//! - Providing group information during matching

use std::collections::HashMap;

/// Information about a capture group
#[derive(Debug, Clone, PartialEq)]
pub struct GroupInfo {
    /// The index of the group (1-based for compatibility with \1, \2, etc.)
    pub index: u32,
    /// The name of the group (if it's a named group)
    pub name: Option<String>,
    /// Whether this is a named group
    pub is_named: bool,
}

/// Registry for tracking capture groups
#[derive(Debug, Clone, Default)]
pub struct GroupRegistry {
    /// Map from group index to group info
    groups: Vec<GroupInfo>,
    /// Map from group name to index
    name_to_index: HashMap<String, u32>,
    /// The next group index to assign
    next_index: u32,
}

impl GroupRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        GroupRegistry {
            groups: Vec::new(),
            name_to_index: HashMap::new(),
            next_index: 1, // Groups are 1-indexed
        }
    }

    /// Register a new capture group
    ///
    /// # Arguments
    /// * `name` - Optional name for the group
    ///
    /// # Returns
    /// The index assigned to this group
    ///
    /// # Errors
    /// Returns an error if the name is already in use
    pub fn register_group(&mut self, name: Option<String>) -> Result<u32, GroupRegistryError> {
        let index = self.next_index;
        self.next_index += 1;

        // Check for duplicate names
        if let Some(ref group_name) = name {
            if self.name_to_index.contains_key(group_name) {
                return Err(GroupRegistryError::DuplicateGroupName(group_name.clone()));
            }
            self.name_to_index.insert(group_name.clone(), index);
        }

        let info = GroupInfo {
            index,
            name: name.clone(),
            is_named: name.is_some(),
        };

        self.groups.push(info);
        Ok(index)
    }

    /// Get group info by index
    pub fn get_by_index(&self, index: u32) -> Option<&GroupInfo> {
        self.groups.iter().find(|g| g.index == index)
    }

    /// Get group index by name
    pub fn get_by_name(&self, name: &str) -> Option<u32> {
        self.name_to_index.get(name).copied()
    }

    /// Check if a group name exists
    pub fn has_name(&self, name: &str) -> bool {
        self.name_to_index.contains_key(name)
    }

    /// Get the total number of capture groups
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Get all group infos
    pub fn groups(&self) -> &[GroupInfo] {
        &self.groups
    }

    /// Validate that a backreference name exists
    pub fn validate_backref_name(&self, name: &str) -> Result<u32, GroupRegistryError> {
        self.get_by_name(name)
            .ok_or_else(|| GroupRegistryError::UndefinedBackreference(name.to_string()))
    }

    /// Validate that a backreference number exists
    pub fn validate_backref_number(&self, num: u32) -> Result<u32, GroupRegistryError> {
        if num == 0 || num >= self.next_index {
            Err(GroupRegistryError::InvalidBackreference(num))
        } else {
            Ok(num)
        }
    }
}

/// Errors that can occur in the group registry
#[derive(Debug, Clone, PartialEq)]
pub enum GroupRegistryError {
    /// A group name is used more than once
    DuplicateGroupName(String),
    /// A backreference refers to a non-existent group
    UndefinedBackreference(String),
    /// A backreference number is invalid
    InvalidBackreference(u32),
}

impl std::fmt::Display for GroupRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupRegistryError::DuplicateGroupName(name) => {
                write!(f, "duplicate group name: {}", name)
            }
            GroupRegistryError::UndefinedBackreference(name) => {
                write!(f, "undefined backreference: {}", name)
            }
            GroupRegistryError::InvalidBackreference(num) => {
                write!(f, "invalid backreference number: {}", num)
            }
        }
    }
}

impl std::error::Error for GroupRegistryError {}

/// A visitor that collects group information from an AST
pub struct GroupCollector;

impl GroupCollector {
    /// Collect all groups from an expression and populate the registry
    pub fn collect(
        expr: &crate::ast::Expr,
        registry: &mut GroupRegistry,
    ) -> Result<(), GroupRegistryError> {
        Self::visit_expr(expr, registry)
    }

    fn visit_expr(
        expr: &crate::ast::Expr,
        registry: &mut GroupRegistry,
    ) -> Result<(), GroupRegistryError> {
        match expr {
            crate::ast::Expr::Empty
            | crate::ast::Expr::Literal(_)
            | crate::ast::Expr::Any
            | crate::ast::Expr::StartAnchor
            | crate::ast::Expr::EndAnchor
            | crate::ast::Expr::Backreference(_)
            | crate::ast::Expr::NamedBackreference(_) => Ok(()),

            crate::ast::Expr::Sequence(exprs) => {
                for expr in exprs {
                    Self::visit_expr(expr, registry)?;
                }
                Ok(())
            }

            crate::ast::Expr::Alternation(exprs) => {
                for expr in exprs {
                    Self::visit_expr(expr, registry)?;
                }
                Ok(())
            }

            crate::ast::Expr::Quantified { expr, .. } => Self::visit_expr(expr, registry),

            crate::ast::Expr::Group(expr) => {
                registry.register_group(None)?;
                Self::visit_expr(expr, registry)
            }

            crate::ast::Expr::NonCapturingGroup(expr) => {
                // Non-capturing groups don't register
                Self::visit_expr(expr, registry)
            }

            crate::ast::Expr::NamedGroup { name, pattern } => {
                registry.register_group(Some(name.clone()))?;
                Self::visit_expr(pattern, registry)
            }

            crate::ast::Expr::CharacterClass(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_unnamed_group() {
        let mut registry = GroupRegistry::new();
        let index = registry.register_group(None).unwrap();
        assert_eq!(index, 1);
        assert_eq!(registry.group_count(), 1);
    }

    #[test]
    fn test_register_named_group() {
        let mut registry = GroupRegistry::new();
        let index = registry.register_group(Some("name".to_string())).unwrap();
        assert_eq!(index, 1);
        assert!(registry.has_name("name"));
        assert_eq!(registry.get_by_name("name"), Some(1));
    }

    #[test]
    fn test_register_multiple_groups() {
        let mut registry = GroupRegistry::new();
        let idx1 = registry.register_group(Some("first".to_string())).unwrap();
        let idx2 = registry.register_group(None).unwrap();
        let idx3 = registry.register_group(Some("third".to_string())).unwrap();

        assert_eq!(idx1, 1);
        assert_eq!(idx2, 2);
        assert_eq!(idx3, 3);
        assert_eq!(registry.group_count(), 3);
    }

    #[test]
    fn test_duplicate_name_error() {
        let mut registry = GroupRegistry::new();
        registry.register_group(Some("name".to_string())).unwrap();
        let result = registry.register_group(Some("name".to_string()));
        assert!(matches!(
            result,
            Err(GroupRegistryError::DuplicateGroupName(_))
        ));
    }

    #[test]
    fn test_validate_backref_name() {
        let mut registry = GroupRegistry::new();
        registry.register_group(Some("name".to_string())).unwrap();

        assert_eq!(registry.validate_backref_name("name").unwrap(), 1);
        assert!(matches!(
            registry.validate_backref_name("unknown"),
            Err(GroupRegistryError::UndefinedBackreference(_))
        ));
    }

    #[test]
    fn test_validate_backref_number() {
        let mut registry = GroupRegistry::new();
        registry.register_group(None).unwrap();
        registry.register_group(None).unwrap();

        assert_eq!(registry.validate_backref_number(1).unwrap(), 1);
        assert_eq!(registry.validate_backref_number(2).unwrap(), 2);
        assert!(matches!(
            registry.validate_backref_number(3),
            Err(GroupRegistryError::InvalidBackreference(3))
        ));
        assert!(matches!(
            registry.validate_backref_number(0),
            Err(GroupRegistryError::InvalidBackreference(0))
        ));
    }

    #[test]
    fn test_get_by_index() {
        let mut registry = GroupRegistry::new();
        registry.register_group(Some("name".to_string())).unwrap();

        let info = registry.get_by_index(1).unwrap();
        assert_eq!(info.index, 1);
        assert_eq!(info.name, Some("name".to_string()));
        assert!(info.is_named);
    }

    #[test]
    fn test_collector_with_simple_groups() {
        use crate::ast::Expr;

        let expr = Expr::sequence(vec![
            Expr::group(Expr::literal('a')),
            Expr::named_group("second", Expr::literal('b')),
        ]);

        let mut registry = GroupRegistry::new();
        GroupCollector::collect(&expr, &mut registry).unwrap();

        assert_eq!(registry.group_count(), 2);
        assert!(registry.get_by_index(1).is_some());
        assert_eq!(registry.get_by_name("second"), Some(2));
    }

    #[test]
    fn test_collector_with_nested_groups() {
        use crate::ast::Expr;

        let expr = Expr::named_group(
            "outer",
            Expr::sequence(vec![
                Expr::literal('a'),
                Expr::named_group("inner", Expr::literal('b')),
            ]),
        );

        let mut registry = GroupRegistry::new();
        GroupCollector::collect(&expr, &mut registry).unwrap();

        assert_eq!(registry.group_count(), 2);
        assert_eq!(registry.get_by_name("outer"), Some(1));
        assert_eq!(registry.get_by_name("inner"), Some(2));
    }

    #[test]
    fn test_collector_with_duplicate_names() {
        use crate::ast::Expr;

        let expr = Expr::sequence(vec![
            Expr::named_group("dup", Expr::literal('a')),
            Expr::named_group("dup", Expr::literal('b')),
        ]);

        let mut registry = GroupRegistry::new();
        let result = GroupCollector::collect(&expr, &mut registry);
        assert!(matches!(
            result,
            Err(GroupRegistryError::DuplicateGroupName(_))
        ));
    }
}
