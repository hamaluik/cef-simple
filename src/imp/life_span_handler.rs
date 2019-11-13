use libc::{calloc, free};
use std::mem::size_of;
use std::os::raw::{c_int, c_void};
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

extern "C" fn on_before_close(_slf: *mut cef_life_span_handler_t, _browser: *mut cef_browser_t) {
    unsafe {
        cef_quit_message_loop();
    }
}

pub fn allocate() -> *mut LifeSpanHandler {
    let life_span_handler =
        unsafe { calloc(1, size_of::<LifeSpanHandler>()) as *mut LifeSpanHandler };
    unsafe {
        (*life_span_handler).life_span_handler.base.size = size_of::<LifeSpanHandler>();
        (*life_span_handler).ref_count.store(1, Ordering::SeqCst);
        (*life_span_handler).life_span_handler.base.add_ref = Some(add_ref);
        (*life_span_handler).life_span_handler.base.release = Some(release);
        (*life_span_handler).life_span_handler.base.has_one_ref = Some(has_one_ref);
        (*life_span_handler)
            .life_span_handler
            .base
            .has_at_least_one_ref = Some(has_at_least_one_ref);

        (*life_span_handler).life_span_handler.do_close = Some(do_close);
        (*life_span_handler).life_span_handler.on_before_close = Some(on_before_close);
    };

    life_span_handler
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
            free(life_span_handler as *mut c_void);
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
