use libc::{calloc, free};
use std::mem::size_of;
use std::os::raw::{c_int, c_void};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{
    cef_base_ref_counted_t, cef_browser_t, cef_display_handler_t, cef_log_severity_t,
    cef_log_severity_t_LOGSEVERITY_DEBUG, cef_log_severity_t_LOGSEVERITY_DEFAULT,
    cef_log_severity_t_LOGSEVERITY_ERROR, cef_log_severity_t_LOGSEVERITY_FATAL,
    cef_log_severity_t_LOGSEVERITY_INFO, cef_log_severity_t_LOGSEVERITY_WARNING, cef_string_t,
};

#[repr(C)]
pub struct DisplayHandler {
    display_handler: cef_display_handler_t,
    ref_count: AtomicUsize,
}

impl DisplayHandler {
    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn on_fullscreen_mode_change(
    _slf: *mut cef_display_handler_t,
    _browser: *mut cef_browser_t,
    fullscreen: i32,
) {
    eprintln!("on_fullscreen_mode_change: {}", fullscreen);
}

extern "C" fn on_console_message(
    _slf: *mut cef_display_handler_t,
    _browser: *mut cef_browser_t,
    level: cef_log_severity_t,
    message: *const cef_string_t,
    _source: *const cef_string_t,
    _line: i32,
) -> i32 {
    let chars: *mut u16 = unsafe { (*message).str };
    let len: usize = unsafe { (*message).length };
    let chars = unsafe { std::slice::from_raw_parts(chars, len) };
    let message = std::char::decode_utf16(chars.iter().cloned())
        .map(|r| r.unwrap_or(std::char::REPLACEMENT_CHARACTER))
        .collect::<String>();

    #[allow(non_upper_case_globals)]
    match level {
        cef_log_severity_t_LOGSEVERITY_DEFAULT => log::info!("[CONSOLE] {}", message),
        cef_log_severity_t_LOGSEVERITY_DEBUG => log::debug!("[CONSOLE] {}", message),
        cef_log_severity_t_LOGSEVERITY_INFO => log::info!("[CONSOLE] {}", message),
        cef_log_severity_t_LOGSEVERITY_WARNING => log::warn!("[CONSOLE] {}", message),
        cef_log_severity_t_LOGSEVERITY_ERROR => log::error!("[CONSOLE] {}", message),
        cef_log_severity_t_LOGSEVERITY_FATAL => log::error!("[CONSOLE] {}", message),
        _ => log::info!("[CONSOLE] {}", message),
    }

    1
}

pub fn allocate() -> *mut DisplayHandler {
    let display_handler = unsafe { calloc(1, size_of::<DisplayHandler>()) as *mut DisplayHandler };
    unsafe {
        (*display_handler).display_handler.base.size = size_of::<DisplayHandler>();
        (*display_handler).ref_count.store(1, Ordering::SeqCst);
        (*display_handler).display_handler.base.add_ref = Some(add_ref);
        (*display_handler).display_handler.base.release = Some(release);
        (*display_handler).display_handler.base.has_one_ref = Some(has_one_ref);
        (*display_handler).display_handler.base.has_at_least_one_ref = Some(has_at_least_one_ref);

        (*display_handler).display_handler.on_fullscreen_mode_change =
            Some(on_fullscreen_mode_change);
        (*display_handler).display_handler.on_console_message = Some(on_console_message);
    };

    display_handler
}

extern "C" fn add_ref(base: *mut cef_base_ref_counted_t) {
    let display_handler = base as *mut DisplayHandler;
    unsafe { (*display_handler).ref_count.fetch_add(1, Ordering::SeqCst) };
}

extern "C" fn release(base: *mut cef_base_ref_counted_t) -> c_int {
    let display_handler = base as *mut DisplayHandler;
    let count = unsafe { (*display_handler).ref_count.fetch_sub(1, Ordering::SeqCst) - 1 };

    if count == 0 {
        unsafe {
            free(display_handler as *mut c_void);
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let display_handler = base as *mut DisplayHandler;
    let count = unsafe { (*display_handler).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let display_handler = base as *mut DisplayHandler;
    let count = unsafe { (*display_handler).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
