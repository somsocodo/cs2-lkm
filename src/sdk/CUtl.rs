use std::ffi::{ CStr, CString };
use libc::c_char;
use std::str;

#[derive(Copy, Clone)]
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
    pub fn new(text: &str) -> Self {
        let c_string = CString::new(text).unwrap_or_else(|_| CString::new("").unwrap());
        let mut buffer: [c_char; 512] = [0; 512];
        let bytes = c_string.as_bytes_with_nul();
        let length = bytes.len().min(511);
        for (i, &byte) in bytes[..length].iter().enumerate() {
            buffer[i] = byte as c_char;
        }
        
        CUtlString { text: buffer }
    }

    pub fn to_str(&self) -> &str {
        let c_str = unsafe { CStr::from_ptr(self.text.as_ptr()) };
        c_str.to_str().unwrap_or("")
    }
}

#[derive(Copy, Clone)]
pub struct CUtlVector {
    pub count: u64,
    pub data: u64
}

impl Default for CUtlVector {
    fn default() -> Self {
        Self {
            count: 0,
            data: 0
        }
    }
}