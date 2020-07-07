use std::mem::size_of;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{
    cef_app_t, cef_base_ref_counted_t, cef_browser_process_handler_t, cef_render_process_handler_t,
};
use super::browser_process_handler::{self, BrowserProcessHandler};
use super::render_process_handler::{self, RenderProcessHandler};

#[repr(C)]
pub struct App {
    app: cef_app_t,
    ref_count: AtomicUsize,
    browser_process_handler: *mut BrowserProcessHandler,
    render_process_handler: *mut RenderProcessHandler,
}

impl App {
    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn get_browser_process_handler(
    slf: *mut cef_app_t,
) -> *mut cef_browser_process_handler_t {
    let app = slf as *mut App;
    let handler = unsafe { (*app).browser_process_handler };
    unsafe { (*handler).inc_ref() };
    handler as *mut cef_browser_process_handler_t
}

extern "C" fn get_render_process_handler(slf: *mut cef_app_t) -> *mut cef_render_process_handler_t {
    let app = slf as *mut App;
    let handler = unsafe { (*app).render_process_handler };
    unsafe { (*handler).inc_ref() };
    handler as *mut cef_render_process_handler_t
}

pub fn allocate() -> *mut App {
    let app = App {
        app: cef_app_t {
            base: cef_base_ref_counted_t {
                size: size_of::<App>() as u64,
                add_ref: Some(add_ref),
                release: Some(release),
                has_one_ref: Some(has_one_ref),
                has_at_least_one_ref: Some(has_at_least_one_ref),
            },
            on_before_command_line_processing: None,
            on_register_custom_schemes: None,
            get_resource_bundle_handler: None,
            get_browser_process_handler: Some(get_browser_process_handler),
            get_render_process_handler: Some(get_render_process_handler),
        },
        ref_count: AtomicUsize::new(1),
        browser_process_handler: browser_process_handler::allocate(),
        render_process_handler: render_process_handler::allocate(),
    };

    Box::into_raw(Box::from(app))
}

extern "C" fn add_ref(base: *mut cef_base_ref_counted_t) {
    let app = base as *mut App;
    unsafe {
        (*app).ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn release(base: *mut cef_base_ref_counted_t) -> i32 {
    let app = base as *mut App;
    let count = unsafe { (*app).ref_count.fetch_sub(1, Ordering::SeqCst) - 1 };

    if count == 0 {
        unsafe {
            let app: Box<App> = Box::from_raw(app as *mut App);
            // TODO: free our handlers here too
            Box::from_raw(app.browser_process_handler);
            Box::from_raw(app.render_process_handler);
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref(base: *mut cef_base_ref_counted_t) -> i32 {
    let app = base as *mut App;
    let count = unsafe { (*app).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref(base: *mut cef_base_ref_counted_t) -> i32 {
    let app = base as *mut App;
    let count = unsafe { (*app).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
