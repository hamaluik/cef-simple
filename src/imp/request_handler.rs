use libc::{calloc, free};
use std::mem::size_of;
use std::os::raw::{c_int, c_void};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{
    cef_base_ref_counted_t, cef_request_handler_t,
};

#[derive(Debug)]
#[repr(C)]
pub struct RequestHandler {
    request_handler: cef_request_handler_t,
    ref_count: AtomicUsize,
}

impl RequestHandler {
    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

pub fn allocate() -> *mut RequestHandler {
    let request_handler = unsafe { calloc(1, size_of::<RequestHandler>()) as *mut RequestHandler };
    unsafe {
        (*request_handler).request_handler.base.size = size_of::<RequestHandler>();
        (*request_handler).ref_count.store(1, Ordering::SeqCst);
        (*request_handler).request_handler.base.add_ref = Some(add_ref);
        (*request_handler).request_handler.base.release = Some(release);
        (*request_handler).request_handler.base.has_one_ref = Some(has_one_ref);
        (*request_handler).request_handler.base.has_at_least_one_ref = Some(has_at_least_one_ref);
    };

    request_handler
}

extern "C" fn add_ref(base: *mut cef_base_ref_counted_t) {
    let request_handler = base as *mut RequestHandler;
    unsafe {
        (*request_handler).ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn release(base: *mut cef_base_ref_counted_t) -> c_int {
    let request_handler = base as *mut RequestHandler;
    let count = unsafe { (*request_handler).ref_count.fetch_sub(1, Ordering::SeqCst) - 1 };

    if count == 0 {
        unsafe {
            free(request_handler as *mut c_void);
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let request_handler = base as *mut RequestHandler;
    let count = unsafe { (*request_handler).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let request_handler = base as *mut RequestHandler;
    let count = unsafe { (*request_handler).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
