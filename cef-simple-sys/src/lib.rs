#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl From<&cef_string_t> for String {
    fn from(s: &cef_string_t) -> String {
        let chars = unsafe { std::slice::from_raw_parts(s.str_, s.length as usize) };
        String::from_utf16_lossy(chars)
    }
}

impl From<String> for cef_string_t {
    fn from(s: String) -> cef_string_t {
        unsafe {
            let mut o = cef_string_t::default();
            cef_string_utf8_to_utf16(s.as_ptr() as *const i8, s.as_bytes().len() as u64, &mut o);
            o
        }
    }
}
