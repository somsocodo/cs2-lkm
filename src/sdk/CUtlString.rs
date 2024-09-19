use std::ffi::CStr;
use libc::c_char;
use std::str;

pub struct CUtlString {
    text: [c_char; 512]
}

impl Default for CUtlString {
    fn default() -> Self {
        Self {
            text: [0; 512]
        }
    }
}

impl CUtlString {
    pub fn to_str(&self) -> &str {
        let c_str = unsafe { CStr::from_ptr(self.text.as_ptr()) };
        c_str.to_str().unwrap_or("")
    }
}