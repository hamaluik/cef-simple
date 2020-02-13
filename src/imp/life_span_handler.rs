use std::mem::size_of;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{
    cef_base_ref_counted_t, cef_browser_t, cef_life_span_handler_t, cef_quit_message_loop,
};

#[repr(C)]
pub struct LifeSpanHandler {
    life_span_handler: cef_life_span_handler_t,
    ref_count: AtomicUsize,
}

impl LifeSpanHandler {
    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn do_close(_slf: *mut cef_life_span_handler_t, _browser: *mut cef_browser_t) -> c_int {
    0
}

unsafe extern "C" fn on_before_close(
    _slf: *mut cef_life_span_handler_t,
    _browser: *mut cef_browser_t,
) {
    cef_quit_message_loop();
}

pub fn allocate() -> *mut LifeSpanHandler {
    let handler = LifeSpanHandler {
        life_span_handler: cef_life_span_handler_t {
            base: cef_base_ref_counted_t {
                size: size_of::<LifeSpanHandler>() as u64,
                add_ref: Some(add_ref),
                release: Some(release),
                has_one_ref: Some(has_one_ref),
                has_at_least_one_ref: Some(has_at_least_one_ref),
            },
            on_before_popup: None,
            on_after_created: None,
            do_close: Some(do_close),
            on_before_close: Some(on_before_close),
        },
        ref_count: AtomicUsize::new(1),
    };

    Box::into_raw(Box::from(handler))
}

extern "C" fn add_ref(base: *mut cef_base_ref_counted_t) {
    let life_span_handler = base as *mut LifeSpanHandler;
    unsafe {
        (*life_span_handler)
            .ref_count
            .fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn release(base: *mut cef_base_ref_counted_t) -> c_int {
    let life_span_handler = base as *mut LifeSpanHandler;
    let count = unsafe {
        (*life_span_handler)
            .ref_count
            .fetch_sub(1, Ordering::SeqCst)
            - 1
    };

    if count == 0 {
        unsafe {
            Box::from_raw(life_span_handler);
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let life_span_handler = base as *mut LifeSpanHandler;
    let count = unsafe { (*life_span_handler).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let life_span_handler = base as *mut LifeSpanHandler;
    let count = unsafe { (*life_span_handler).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
