use libc::{calloc, free};
use std::mem::size_of;
use std::os::raw::{c_int, c_void};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{
    cef_base_ref_counted_t, cef_client_t, cef_context_menu_handler_t, cef_life_span_handler_t,
    cef_request_handler_t,
};
use super::context_menu_handler::{self, ContextMenuHandler};
use super::life_span_handler::{self, LifeSpanHandler};
use super::request_handler::{self, RequestHandler};

#[derive(Debug)]
#[repr(C)]
pub struct Client {
    client: cef_client_t,
    ref_count: AtomicUsize,
    life_span_handler: *mut LifeSpanHandler,
    context_menu_handler: *mut ContextMenuHandler,
    request_handler: *mut RequestHandler,
}

impl Client {
    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn get_life_span_handler(slf: *mut cef_client_t) -> *mut cef_life_span_handler_t {
    let client = slf as *mut Client;
    let handler = unsafe { (*client).life_span_handler };
    handler as *mut cef_life_span_handler_t
}

extern "C" fn get_context_menu_handler(slf: *mut cef_client_t) -> *mut cef_context_menu_handler_t {
    let client = slf as *mut Client;
    let handler = unsafe { (*client).context_menu_handler };
    handler as *mut cef_context_menu_handler_t
}

extern "C" fn get_request_handler(slf: *mut cef_client_t) -> *mut cef_request_handler_t {
    let client = slf as *mut Client;
    let handler = unsafe { (*client).request_handler };
    handler as *mut cef_request_handler_t
}

pub fn allocate() -> *mut Client {
    let client = unsafe { calloc(1, size_of::<Client>()) as *mut Client };
    unsafe {
        (*client).client.base.size = size_of::<Client>();
        (*client).ref_count.store(1, Ordering::SeqCst);
        (*client).client.base.add_ref = Some(add_ref);
        (*client).client.base.release = Some(release);
        (*client).client.base.has_one_ref = Some(has_one_ref);
        (*client).client.base.has_at_least_one_ref = Some(has_at_least_one_ref);

        (*client).life_span_handler = life_span_handler::allocate();
        (*client).client.get_life_span_handler = Some(get_life_span_handler);

        (*client).context_menu_handler = context_menu_handler::allocate();
        (*client).client.get_context_menu_handler = Some(get_context_menu_handler);

        (*client).request_handler = request_handler::allocate();
        (*client).client.get_request_handler = Some(get_request_handler);
    };

    client
}

extern "C" fn add_ref(base: *mut cef_base_ref_counted_t) {
    let client = base as *mut Client;
    unsafe {
        (*(*client).life_span_handler).inc_ref();
        (*(*client).context_menu_handler).inc_ref();
        (*(*client).request_handler).inc_ref();
        (*client).ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn release(base: *mut cef_base_ref_counted_t) -> c_int {
    let client = base as *mut Client;
    let count = unsafe { (*client).ref_count.fetch_sub(1, Ordering::SeqCst) - 1 };

    if count == 0 {
        unsafe {
            free(client as *mut c_void);
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let client = base as *mut Client;
    let count = unsafe { (*client).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let client = base as *mut Client;
    let count = unsafe { (*client).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
