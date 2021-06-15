use std::mem::size_of;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicUsize, Ordering};

use cef_simple_sys::{cef_base_ref_counted_t, cef_request_handler_t};

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
    let handler = RequestHandler {
        request_handler: cef_request_handler_t {
            base: cef_base_ref_counted_t {
                size: size_of::<RequestHandler>() as u64,
                add_ref: Some(add_ref),
                release: Some(release),
                has_one_ref: Some(has_one_ref),
                has_at_least_one_ref: Some(has_at_least_one_ref),
            },
            on_before_browse: None,
            on_open_urlfrom_tab: None,
            get_resource_request_handler: None,
            get_auth_credentials: None,
            on_quota_request: None,
            on_certificate_error: None,
            on_select_client_certificate: None,
            on_plugin_crashed: None,
            on_render_view_ready: None,
            on_render_process_terminated: None,
            on_document_available_in_main_frame: None,
        },
        ref_count: AtomicUsize::new(1),
    };

    Box::into_raw(Box::from(handler))
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
            Box::from_raw(request_handler);
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
