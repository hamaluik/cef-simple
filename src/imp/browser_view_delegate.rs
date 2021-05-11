use std::mem::size_of;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{cef_base_ref_counted_t, cef_browser_view_delegate_t, cef_view_delegate_t};

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
    let browser_view = BrowserViewDelegate {
        browser_view_delegate: cef_browser_view_delegate_t {
            base: cef_view_delegate_t {
                base: cef_base_ref_counted_t {
                    size: size_of::<BrowserViewDelegate>() as u64,
                    add_ref: Some(add_ref),
                    release: Some(release),
                    has_one_ref: Some(has_one_ref),
                    has_at_least_one_ref: Some(has_at_least_one_ref),
                },
                get_preferred_size: None,
                get_minimum_size: None,
                get_maximum_size: None,
                get_height_for_width: None,
                on_parent_view_changed: None,
                on_child_view_changed: None,
                on_focus: None,
                on_blur: None,
                on_window_changed: None,
            },
            on_browser_created: None,
            on_browser_destroyed: None,
            get_delegate_for_popup_browser_view: None,
            on_popup_browser_view_created: None,
            get_chrome_toolbar_type: None,
        },
        ref_count: AtomicUsize::new(1),
    };

    Box::into_raw(Box::from(browser_view))
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
            Box::from_raw(browser_view_delegate as *mut BrowserViewDelegate);
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
