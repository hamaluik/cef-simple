use std::mem::size_of;
use std::os::raw::{c_int};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{
    cef_base_ref_counted_t, cef_browser_process_handler_t, cef_print_handler_t
};
use super::print_handler::{self, PrintHandler};

#[repr(C)]
pub struct BrowserProcessHandler {
    handler: cef_browser_process_handler_t,
    ref_count: AtomicUsize,
    print_handler: *mut PrintHandler,
}

impl BrowserProcessHandler {
    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

unsafe extern "C" fn get_print_handler(slf: *mut cef_browser_process_handler_t) -> *mut cef_print_handler_t {
    let _self = slf as *mut BrowserProcessHandler;
    (*_self).print_handler as *mut cef_print_handler_t
}

pub fn allocate() -> *mut BrowserProcessHandler {
    let handler = BrowserProcessHandler {
        handler: cef_browser_process_handler_t {
            base: cef_base_ref_counted_t {
                size: size_of::<BrowserProcessHandler>() as u64,
                add_ref: Some(add_ref),
                release: Some(release),
                has_one_ref: Some(has_one_ref),
                has_at_least_one_ref: Some(has_at_least_one_ref),
            },
            on_context_initialized: None,
            on_before_child_process_launch: None,
            on_render_process_thread_created: None,
            get_print_handler: Some(get_print_handler),
            on_schedule_message_pump_work: None,
        },
        ref_count: AtomicUsize::new(1),
        print_handler: print_handler::allocate(),
    };

    Box::into_raw(Box::from(handler))
}

extern "C" fn add_ref(base: *mut cef_base_ref_counted_t) {
    let browser_process_handler = base as *mut BrowserProcessHandler;
    unsafe {
        (*browser_process_handler)
            .ref_count
            .fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn release(base: *mut cef_base_ref_counted_t) -> c_int {
    let browser_process_handler = base as *mut BrowserProcessHandler;
    let count = unsafe {
        (*browser_process_handler)
            .ref_count
            .fetch_sub(1, Ordering::SeqCst)
            - 1
    };

    if count == 0 {
        unsafe {
            let app: Box<BrowserProcessHandler> = Box::from_raw(browser_process_handler);
            // TODO: free our handlers here too
            Box::from_raw(app.print_handler);
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let browser_process_handler = base as *mut BrowserProcessHandler;
    let count = unsafe { (*browser_process_handler).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let browser_process_handler = base as *mut BrowserProcessHandler;
    let count = unsafe { (*browser_process_handler).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
