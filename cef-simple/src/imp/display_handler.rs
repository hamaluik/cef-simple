use std::mem::size_of;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicUsize, Ordering};

use cef_simple_sys::{
    cef_base_ref_counted_t, cef_browser_t, cef_display_handler_t, cef_log_severity_t,
    cef_log_severity_t_LOGSEVERITY_DEBUG, cef_log_severity_t_LOGSEVERITY_DEFAULT,
    cef_log_severity_t_LOGSEVERITY_ERROR, cef_log_severity_t_LOGSEVERITY_FATAL,
    cef_log_severity_t_LOGSEVERITY_INFO, cef_log_severity_t_LOGSEVERITY_WARNING, cef_string_t,
    cef_window_t,
};

#[repr(C)]
pub struct DisplayHandler {
    display_handler: cef_display_handler_t,
    ref_count: AtomicUsize,
    window: *mut cef_window_t,
}

impl DisplayHandler {
    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

unsafe extern "C" fn on_fullscreen_mode_change(
    slf: *mut cef_display_handler_t,
    _browser: *mut cef_browser_t,
    fullscreen: i32,
) {
    log::trace!("on_fullscreen_mode_change");
    let handler = slf as *mut DisplayHandler;
    (*(*handler).window)
        .set_fullscreen
        .expect("set_fullscreen exists")((*handler).window, fullscreen);
}

unsafe extern "C" fn on_tooltip(
    _slf: *mut cef_display_handler_t,
    _browser: *mut cef_browser_t,
    _text: *mut cef_string_t,
) -> i32 {
    log::trace!("on_tooltip");
    1
}

extern "C" fn on_console_message(
    _slf: *mut cef_display_handler_t,
    _browser: *mut cef_browser_t,
    level: cef_log_severity_t,
    message: *const cef_string_t,
    _source: *const cef_string_t,
    _line: i32,
) -> i32 {
    log::trace!("on_console_message");
    let chars: *mut u16 = unsafe { (*message).str_ };
    let len: usize = unsafe { (*message).length } as usize;
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

pub fn allocate(window: *mut cef_window_t) -> *mut DisplayHandler {
    let handler = DisplayHandler {
        display_handler: cef_display_handler_t {
            base: cef_base_ref_counted_t {
                size: size_of::<DisplayHandler>() as u64,
                add_ref: Some(add_ref),
                release: Some(release),
                has_one_ref: Some(has_one_ref),
                has_at_least_one_ref: Some(has_at_least_one_ref),
            },
            on_address_change: None,
            on_title_change: None,
            on_favicon_urlchange: None,
            on_fullscreen_mode_change: Some(on_fullscreen_mode_change),
            on_tooltip: Some(on_tooltip),
            on_status_message: None,
            on_console_message: Some(on_console_message),
            on_auto_resize: None,
            on_loading_progress_change: None,
            on_cursor_change: None,
        },
        window,
        ref_count: AtomicUsize::new(1),
    };

    Box::into_raw(Box::from(handler))
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
            Box::from_raw(display_handler);
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
