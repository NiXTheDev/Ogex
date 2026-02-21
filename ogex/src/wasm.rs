//! WebAssembly bindings for JavaScript
//!
//! This module provides WASM bindings using wasm-bindgen for use in browsers
//! and Node.js. Enable with the "wasm" feature flag.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use crate::engine::{Match, Regex};

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
    /// Returns an error string if compilation fails
    #[wasm_bindgen(constructor)]
    pub fn new(pattern: &str) -> Result<JsRegex, JsValue> {
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

    /// Transpile pattern to legacy syntax
    #[wasm_bindgen(js_name = transpile)]
    pub fn transpile(pattern: &str) -> Result<String, JsValue> {
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
