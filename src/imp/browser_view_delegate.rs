use libc::{calloc, free};
use std::mem::size_of;
use std::os::raw::{c_int, c_void};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{cef_base_ref_counted_t, cef_browser_view_delegate_t};

#[derive(Debug)]
#[repr(C)]
pub struct BrowserViewDelegate {
    browser_view_delegate: cef_browser_view_delegate_t,
    ref_count: AtomicUsize,
}

impl BrowserViewDelegate {
    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

pub fn allocate() -> *mut BrowserViewDelegate {
    let browser_view_delegate =
        unsafe { calloc(1, size_of::<BrowserViewDelegate>()) as *mut BrowserViewDelegate };
    unsafe {
        (*browser_view_delegate)
            .browser_view_delegate
            .base
            .base
            .size = size_of::<BrowserViewDelegate>();
        (*browser_view_delegate)
            .ref_count
            .store(1, Ordering::SeqCst);
        (*browser_view_delegate)
            .browser_view_delegate
            .base
            .base
            .add_ref = Some(add_ref);
        (*browser_view_delegate)
            .browser_view_delegate
            .base
            .base
            .release = Some(release);
        (*browser_view_delegate)
            .browser_view_delegate
            .base
            .base
            .has_one_ref = Some(has_one_ref);
        (*browser_view_delegate)
            .browser_view_delegate
            .base
            .base
            .has_at_least_one_ref = Some(has_at_least_one_ref);
    };

    browser_view_delegate
}

extern "C" fn add_ref(base: *mut cef_base_ref_counted_t) {
    let browser_view_delegate = base as *mut BrowserViewDelegate;
    unsafe {
        (*browser_view_delegate)
            .ref_count
            .fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn release(base: *mut cef_base_ref_counted_t) -> c_int {
    let browser_view_delegate = base as *mut BrowserViewDelegate;
    let count = unsafe {
        (*browser_view_delegate)
            .ref_count
            .fetch_sub(1, Ordering::SeqCst)
            - 1
    };

    if count == 0 {
        unsafe {
            free(browser_view_delegate as *mut c_void);
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let browser_view_delegate = base as *mut BrowserViewDelegate;
    let count = unsafe { (*browser_view_delegate).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let browser_view_delegate = base as *mut BrowserViewDelegate;
    let count = unsafe { (*browser_view_delegate).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
