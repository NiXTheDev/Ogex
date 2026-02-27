//! C API for FFI bindings
//!
//! This module provides a C-compatible interface for using the regex engine
//! from other languages via FFI. All functions are marked with #[unsafe(no_mangle)]
//! and use C calling conventions.

use crate::engine::{Match, Regex};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// Opaque handle to a compiled regex
pub struct RegexHandle {
    regex: Regex,
}

/// Opaque handle to a match result
pub struct MatchHandle {
    match_result: Match,
    input: String, // Keep input alive for string references
}

/// Compile a regex pattern
///
/// # Safety
/// - pattern must be a valid null-terminated UTF-8 string
/// - error pointer can be null if you don't need error messages
///
/// Returns a handle to the compiled regex, or null on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ogex_compile(
    pattern: *const c_char,
    error: *mut *mut c_char,
) -> *mut RegexHandle {
    unsafe {
        if pattern.is_null() {
            if !error.is_null() {
                let err = CString::new("pattern is null").unwrap();
                *error = err.into_raw();
            }
            return std::ptr::null_mut();
        }

        let pattern_str = match CStr::from_ptr(pattern).to_str() {
            Ok(s) => s,
            Err(_) => {
                if !error.is_null() {
                    let err = CString::new("pattern is not valid UTF-8").unwrap();
                    *error = err.into_raw();
                }
                return std::ptr::null_mut();
            }
        };

        match Regex::new(pattern_str) {
            Ok(regex) => {
                let handle = Box::new(RegexHandle { regex });
                Box::into_raw(handle)
            }
            Err(e) => {
                if !error.is_null() {
                    let err = CString::new(e.to_string()).unwrap();
                    *error = err.into_raw();
                }
                std::ptr::null_mut()
            }
        }
    }
}

/// Free a regex handle
///
/// # Safety
/// - handle must be a valid pointer returned by ogex_compile
/// - handle must not be used after calling this function
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ogex_free_regex(handle: *mut RegexHandle) {
    unsafe {
        if !handle.is_null() {
            drop(Box::from_raw(handle));
        }
    }
}

/// Check if a pattern matches
///
/// # Safety
/// - handle must be a valid regex handle
/// - input must be a valid null-terminated UTF-8 string
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ogex_is_match(handle: *const RegexHandle, input: *const c_char) -> c_int {
    unsafe {
        if handle.is_null() || input.is_null() {
            return 0;
        }

        let input_str = match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let regex = &(*handle).regex;
        if regex.is_match(input_str) { 1 } else { 0 }
    }
}

/// Find the first match
///
/// # Safety
/// - handle must be a valid regex handle
/// - input must be a valid null-terminated UTF-8 string
/// - error pointer can be null
///
/// Returns a match handle, or null if no match found
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ogex_find(
    handle: *const RegexHandle,
    input: *const c_char,
    error: *mut *mut c_char,
) -> *mut MatchHandle {
    unsafe {
        if handle.is_null() || input.is_null() {
            if !error.is_null() {
                let err = CString::new("invalid handle or input").unwrap();
                *error = err.into_raw();
            }
            return std::ptr::null_mut();
        }

        let input_str = match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => {
                if !error.is_null() {
                    let err = CString::new("input is not valid UTF-8").unwrap();
                    *error = err.into_raw();
                }
                return std::ptr::null_mut();
            }
        };

        let regex = &(*handle).regex;
        match regex.find(input_str) {
            Some(match_result) => {
                let handle = Box::new(MatchHandle {
                    match_result,
                    input: input_str.to_string(),
                });
                Box::into_raw(handle)
            }
            None => std::ptr::null_mut(),
        }
    }
}

/// Free a match handle
///
/// # Safety
/// - handle must be a valid pointer returned by ogex_find
/// - handle must not be used after calling this function
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ogex_free_match(handle: *mut MatchHandle) {
    unsafe {
        if !handle.is_null() {
            drop(Box::from_raw(handle));
        }
    }
}

/// Get match start position
///
/// # Safety
/// - handle must be a valid match handle
/// - returns -1 if handle is null
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ogex_match_start(handle: *const MatchHandle) -> c_int {
    unsafe {
        if handle.is_null() {
            return -1;
        }
        (*handle).match_result.start as c_int
    }
}

/// Get match end position
///
/// # Safety
/// - handle must be a valid match handle
/// - returns -1 if handle is null
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ogex_match_end(handle: *const MatchHandle) -> c_int {
    unsafe {
        if handle.is_null() {
            return -1;
        }
        (*handle).match_result.end as c_int
    }
}

/// Get the matched text
///
/// # Safety
/// - handle must be a valid match handle
/// - returns null if handle is null
/// - caller must free the returned string with ogex_free_string
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ogex_match_text(handle: *const MatchHandle) -> *mut c_char {
    unsafe {
        if handle.is_null() {
            return std::ptr::null_mut();
        }

        let m = &(*handle).match_result;
        let text = m.as_str(&(*handle).input);
        match CString::new(text) {
            Ok(cstr) => cstr.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    }
}

/// Free a string returned by the API
///
/// # Safety
/// - ptr must be a valid pointer returned by this API
/// - ptr must not be used after calling this function
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ogex_free_string(ptr: *mut c_char) {
    unsafe {
        if !ptr.is_null() {
            drop(CString::from_raw(ptr));
        }
    }
}

/// Free an error string
///
/// # Safety
/// - ptr must be a valid error pointer returned by this API
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ogex_free_error(ptr: *mut c_char) {
    unsafe {
        ogex_free_string(ptr);
    }
}

/// Get the API version
#[unsafe(no_mangle)]
pub extern "C" fn ogex_version() -> *const c_char {
    const VERSION: &[u8] = b"0.1.0\0";
    VERSION.as_ptr() as *const c_char
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_api_compile_and_match() {
        unsafe {
            let pattern = CString::new("hello").unwrap();
            let mut error: *mut c_char = std::ptr::null_mut();

            let regex = ogex_compile(pattern.as_ptr(), &mut error);
            assert!(!regex.is_null());

            let input = CString::new("hello world").unwrap();
            let matched = ogex_is_match(regex, input.as_ptr());
            assert_eq!(matched, 1);

            let input2 = CString::new("goodbye").unwrap();
            let matched2 = ogex_is_match(regex, input2.as_ptr());
            assert_eq!(matched2, 0);

            ogex_free_regex(regex);
        }
    }

    #[test]
    fn test_c_api_find() {
        unsafe {
            let pattern = CString::new("abc").unwrap();
            let mut error: *mut c_char = std::ptr::null_mut();

            let regex = ogex_compile(pattern.as_ptr(), &mut error);
            assert!(!regex.is_null());

            let input = CString::new("xabcy").unwrap();
            let match_handle = ogex_find(regex, input.as_ptr(), &mut error);
            assert!(!match_handle.is_null());

            let start = ogex_match_start(match_handle);
            let end = ogex_match_end(match_handle);
            assert_eq!(start, 1);
            assert_eq!(end, 4);

            let text = ogex_match_text(match_handle);
            assert!(!text.is_null());

            let text_str = CStr::from_ptr(text).to_str().unwrap();
            assert_eq!(text_str, "abc");

            ogex_free_string(text);
            ogex_free_match(match_handle);
            ogex_free_regex(regex);
        }
    }
}
