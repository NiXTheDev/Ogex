//! WebAssembly bindings for JavaScript
//!
//! This module provides WASM bindings using wasm-bindgen for use in browsers
//! and Node.js. Enable with the "wasm" feature flag.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use crate::engine::{Match, Regex};

/// JavaScript-facing structured error
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct JsError {
    /// Error type: "Lexer", "Parser", "Compile", "Runtime"
    error_type: String,
    /// Error message
    message: String,
    /// Position in pattern where error occurred (for lexer/parser errors)
    position: Option<usize>,
    /// Additional context (e.g., invalid character for lexer errors)
    context: Option<String>,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl JsError {
    /// Create a new JS error from a RegexError
    pub fn from_error(e: &crate::RegexError) -> JsError {
        match e {
            crate::RegexError::Lexer { position, kind } => JsError {
                error_type: "Lexer".to_string(),
                message: kind.to_string(),
                position: Some(*position),
                context: Some(format!("{:?}", kind)),
            },
            crate::RegexError::Parse(parse_err) => JsError {
                error_type: "Parser".to_string(),
                message: parse_err.to_string(),
                position: None, // TODO: Add span support to parse errors for WASM
                context: None,
            },
            crate::RegexError::Compile(msg) => JsError {
                error_type: "Compile".to_string(),
                message: msg.clone(),
                position: None,
                context: None,
            },
            crate::RegexError::Runtime(msg) => JsError {
                error_type: "Runtime".to_string(),
                message: msg.clone(),
                position: None,
                context: None,
            },
        }
    }

    /// Get error type
    #[wasm_bindgen(getter)]
    pub fn error_type(&self) -> String {
        self.error_type.clone()
    }

    /// Get error message
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// Get error position (if available)
    #[wasm_bindgen(getter)]
    pub fn position(&self) -> Option<usize> {
        self.position
    }

    /// Get error context (if available)
    #[wasm_bindgen(getter)]
    pub fn context(&self) -> Option<String> {
        self.context.clone()
    }

    /// Convert to JSON string for easy debugging
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> String {
        let pos = self
            .position
            .map(|p| p.to_string())
            .unwrap_or_else(|| "null".to_string());
        let ctx = self
            .context
            .clone()
            .map(|c| format!("\"{}\"", c))
            .unwrap_or_else(|| "null".to_string());
        format!(
            r#"{{"error_type":"{}","message":"{}","position":{},"context":{}}}"#,
            self.error_type, self.message, pos, ctx
        )
    }
}

/// JavaScript-facing regex wrapper
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct JsRegex {
    regex: Regex,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl JsRegex {
    /// Compile a regex pattern
    ///
    /// Returns a JsError if compilation fails
    #[wasm_bindgen(constructor)]
    pub fn new(pattern: &str) -> Result<JsRegex, JsError> {
        match Regex::new(pattern) {
            Ok(regex) => Ok(JsRegex { regex }),
            Err(e) => Err(JsError::from_error(&e)),
        }
    }

    /// Compile a regex pattern (legacy string-based error)
    ///
    /// Returns an error string if compilation fails
    #[wasm_bindgen(js_name = newWithStringError)]
    pub fn new_with_string_error(pattern: &str) -> Result<JsRegex, JsValue> {
        match Regex::new(pattern) {
            Ok(regex) => Ok(JsRegex { regex }),
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Check if the pattern matches anywhere in the input
    #[wasm_bindgen(js_name = isMatch)]
    pub fn is_match(&self, input: &str) -> bool {
        self.regex.is_match(input)
    }

    /// Find the first match
    #[wasm_bindgen(js_name = find)]
    pub fn find(&self, input: &str) -> Option<JsMatch> {
        self.regex.find(input).map(|m| JsMatch {
            match_result: m,
            input: input.to_string(),
        })
    }

    /// Find all matches
    #[wasm_bindgen(js_name = findAll)]
    pub fn find_all(&self, input: &str) -> js_sys::Array {
        let matches = self.regex.find_all(input);
        let array = js_sys::Array::new();

        for m in matches {
            array.push(
                &JsMatch {
                    match_result: m,
                    input: input.to_string(),
                }
                .into(),
            );
        }

        array
    }

    /// Transpile pattern to legacy syntax (structured error)
    #[wasm_bindgen(js_name = transpile)]
    pub fn transpile(pattern: &str) -> Result<String, JsError> {
        match crate::transpile(pattern) {
            Ok(result) => Ok(result),
            Err(e) => Err(JsError::from_error(&e)),
        }
    }

    /// Transpile pattern to legacy syntax (string error - legacy)
    #[wasm_bindgen(js_name = transpileWithStringError)]
    pub fn transpile_with_string_error(pattern: &str) -> Result<String, JsValue> {
        match crate::transpile(pattern) {
            Ok(result) => Ok(result),
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }
}

/// JavaScript-facing match result wrapper
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct JsMatch {
    match_result: Match,
    input: String,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl JsMatch {
    /// Get start position
    #[wasm_bindgen(getter)]
    pub fn start(&self) -> usize {
        self.match_result.start
    }

    /// Get end position
    #[wasm_bindgen(getter)]
    pub fn end(&self) -> usize {
        self.match_result.end
    }

    /// Get matched text
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        self.match_result.as_str(&self.input).to_string()
    }

    /// Get capture group by index
    #[wasm_bindgen(js_name = group)]
    pub fn group(&self, index: u32) -> Option<String> {
        self.match_result
            .group(index)
            .map(|(start, end)| self.input[start..end].to_string())
    }

    /// Get named capture group
    #[wasm_bindgen(js_name = namedGroup)]
    pub fn named_group(&self, name: &str) -> Option<String> {
        self.match_result
            .named_group(name)
            .map(|(start, end)| self.input[start..end].to_string())
    }

    /// Get all groups as a JavaScript object
    #[wasm_bindgen(getter, js_name = groups)]
    pub fn groups(&self) -> js_sys::Object {
        let obj = js_sys::Object::new();

        for (idx, (start, end)) in &self.match_result.groups {
            let text = &self.input[*start..*end];
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_f64(*idx as f64),
                &JsValue::from_str(text),
            )
            .unwrap();
        }

        obj
    }
}

/// Initialize panic hook for better error messages in WASM
#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[cfg(all(test, feature = "wasm"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_wasm_regex_new() {
        let regex = JsRegex::new("abc").unwrap();
        assert!(regex.is_match("abc"));
        assert!(!regex.is_match("def"));
    }

    #[wasm_bindgen_test]
    fn test_wasm_regex_find() {
        let regex = JsRegex::new("abc").unwrap();
        let m = regex.find("xabcy").unwrap();
        assert_eq!(m.start(), 1);
        assert_eq!(m.end(), 4);
        assert_eq!(m.text(), "abc");
    }

    #[wasm_bindgen_test]
    fn test_wasm_transpile() {
        let result = JsRegex::transpile("(name:abc)").unwrap();
        assert_eq!(result, "(?<name>abc)");
    }
}
