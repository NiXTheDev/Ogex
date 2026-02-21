//! Python bindings for Ogex regex engine
//!
//! This module provides Python bindings for the Ogex regex engine,
//! offering a `re`-compatible API with Ogex's unified syntax.

use pyo3::prelude::*;
use pyo3::types::PyList;
use std::collections::HashMap as StdHashMap;

/// A compiled regex pattern
#[pyclass(name = "Regex")]
pub struct PyRegex {
    inner: ogex_lib::Regex,
}

#[pymethods]
impl PyRegex {
    /// Compile a regex pattern
    #[new]
    fn new(pattern: &str) -> PyResult<Self> {
        let regex = ogex_lib::Regex::new(pattern)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(PyRegex { inner: regex })
    }

    /// Check if the pattern matches at the beginning of the string
    fn match_(&self, string: &str) -> Option<PyMatch> {
        // Check if match is at position 0
        if let Some(m) = self.inner.find(string)
            && m.start == 0
        {
            return Some(PyMatch::new(m, string.to_string()));
        }
        None
    }

    /// Search for a match anywhere in the string
    fn search(&self, string: &str) -> Option<PyMatch> {
        self.inner
            .find(string)
            .map(|m| PyMatch::new(m, string.to_string()))
    }

    /// Check if the pattern matches the string
    fn is_match(&self, string: &str) -> bool {
        self.inner.is_match(string)
    }

    /// Find all non-overlapping matches
    fn findall<'py>(&self, py: Python<'py>, string: &str) -> PyResult<Bound<'py, PyList>> {
        let matches: Vec<_> = self.inner.find_all(string);

        let list = PyList::empty(py);
        for m in matches {
            let py_match = PyMatch::new(m, string.to_string());
            list.append(py_match)?;
        }
        Ok(list)
    }

    /// Replace matches with a replacement string
    #[pyo3(signature = (repl, string, count=None))]
    fn sub(&self, repl: &str, string: &str, count: Option<usize>) -> PyResult<String> {
        let replacement = ogex_lib::Replacement::parse(repl)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        let max_replacements = count.unwrap_or(usize::MAX);
        let mut result = String::new();
        let mut last_end = 0;

        for (i, m) in self.inner.find_all(string).into_iter().enumerate() {
            if i >= max_replacements {
                break;
            }

            // Add text before match
            result.push_str(&string[last_end..m.start]);

            // Get groups as pairs in order
            let mut group_pairs = vec![(0usize, 0usize); m.groups.len()];
            for (&idx, &(s, e)) in &m.groups {
                if idx > 0 && (idx as usize) <= group_pairs.len() {
                    group_pairs[(idx - 1) as usize] = (s, e);
                }
            }

            // Named groups mapping (empty for now)
            let named = StdHashMap::new();

            // Apply replacement
            let replaced =
                replacement.apply_with_names(string, m.start, m.end, &group_pairs, &named);
            result.push_str(&replaced);

            last_end = m.end;
        }

        // Add remaining text
        result.push_str(&string[last_end..]);

        Ok(result)
    }
}

/// A match result
#[pyclass(name = "Match")]
pub struct PyMatch {
    start: usize,
    end: usize,
    groups: StdHashMap<u32, (usize, usize)>,
    input: String,
}

impl PyMatch {
    fn new(m: ogex_lib::Match, input: String) -> Self {
        PyMatch {
            start: m.start,
            end: m.end,
            groups: m.groups,
            input,
        }
    }
}

#[pymethods]
impl PyMatch {
    /// The matched text
    #[getter]
    fn group0(&self) -> &str {
        &self.input[self.start..self.end]
    }

    /// Get a group by index
    fn group(&self, n: u32) -> Option<&str> {
        self.groups.get(&n).map(|(s, e)| &self.input[*s..*e])
    }

    /// Start position of the match
    #[getter]
    fn start(&self) -> usize {
        self.start
    }

    /// End position of the match
    #[getter]
    fn end(&self) -> usize {
        self.end
    }

    /// The matched text
    #[getter]
    fn text(&self) -> &str {
        &self.input[self.start..self.end]
    }

    /// All captured groups as a list
    #[getter]
    fn groups(&self) -> Vec<Option<String>> {
        let max_group = self.groups.keys().max().copied().unwrap_or(0);
        let mut result = Vec::with_capacity(max_group as usize);
        for i in 1..=max_group {
            if let Some(&(s, e)) = self.groups.get(&i) {
                result.push(Some(self.input[s..e].to_string()));
            } else {
                result.push(None);
            }
        }
        result
    }
}

/// Compile a regex pattern
#[pyfunction]
fn compile(pattern: &str) -> PyResult<PyRegex> {
    PyRegex::new(pattern)
}

/// Search for a match
#[pyfunction]
fn search(pattern: &str, string: &str) -> PyResult<Option<PyMatch>> {
    let regex = PyRegex::new(pattern)?;
    Ok(regex.search(string))
}

/// Check if pattern matches at start
#[pyfunction]
fn match_(pattern: &str, string: &str) -> PyResult<Option<PyMatch>> {
    let regex = PyRegex::new(pattern)?;
    Ok(regex.match_(string))
}

/// Find all matches
#[pyfunction]
fn findall<'py>(py: Python<'py>, pattern: &str, string: &str) -> PyResult<Bound<'py, PyList>> {
    let regex = PyRegex::new(pattern)?;
    regex.findall(py, string)
}

/// Substitute matches
#[pyfunction(signature = (pattern, repl, string, count=None))]
fn sub(pattern: &str, repl: &str, string: &str, count: Option<usize>) -> PyResult<String> {
    let regex = PyRegex::new(pattern)?;
    regex.sub(repl, string, count)
}

/// Ogex Python module
#[pymodule(name = "ogex")]
fn ogex(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyRegex>()?;
    m.add_class::<PyMatch>()?;
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(search, m)?)?;
    m.add_function(wrap_pyfunction!(match_, m)?)?;
    m.add_function(wrap_pyfunction!(findall, m)?)?;
    m.add_function(wrap_pyfunction!(sub, m)?)?;
    Ok(())
}
