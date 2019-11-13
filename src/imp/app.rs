use libc::{calloc, free};
use std::mem::size_of;
use std::os::raw::{c_int, c_void};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{cef_app_t, cef_base_ref_counted_t};

#[derive(Debug)]
#[repr(C)]
pub struct App {
    pub app: cef_app_t,
    pub ref_count: AtomicUsize,
}

impl App {
    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

pub fn allocate() -> *mut App {
    let app = unsafe { calloc(1, size_of::<App>()) as *mut App };
    unsafe {
        (*app).app.base.size = size_of::<App>();
        (*app).ref_count.store(1, Ordering::SeqCst);
        (*app).app.base.add_ref = Some(add_ref);
        (*app).app.base.release = Some(release);
        (*app).app.base.has_one_ref = Some(has_one_ref);
        (*app).app.base.has_at_least_one_ref = Some(has_at_least_one_ref);
    };

    app
}

extern "C" fn add_ref(base: *mut cef_base_ref_counted_t) {
    let app = base as *mut App;
    unsafe {
        (*app).ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn release(base: *mut cef_base_ref_counted_t) -> c_int {
    let app = base as *mut App;
    let count = unsafe { (*app).ref_count.fetch_sub(1, Ordering::SeqCst) - 1 };

    if count == 0 {
        unsafe {
            free(app as *mut c_void);
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let app = base as *mut App;
    let count = unsafe { (*app).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let app = base as *mut App;
    let count = unsafe { (*app).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
