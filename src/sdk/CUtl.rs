use std::ffi::CString;
use libc::c_char;
use std::str;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CUtlString {
    pub text: [c_char; 512]
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

    pub fn to_string(&self) -> String {
        let mut chars = Vec::new();
        let mut ptr = self.text.as_ptr();
    
        unsafe {
            while *ptr != 0 {
                if *ptr != -1 as c_char {
                    chars.push(*ptr as u8);
                }
                ptr = ptr.add(1);
            }
        }
    
        String::from_utf8(chars).unwrap_or_else(|_| String::new())
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