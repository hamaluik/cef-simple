use std::mem::size_of;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{cef_base_ref_counted_t, cef_print_handler_t, cef_size_t};

#[derive(Debug)]
#[repr(C)]
pub struct PrintHandler {
    print_handler: cef_print_handler_t,
    ref_count: AtomicUsize,
}

unsafe extern "C" fn get_pdf_paper_size(
    _slf: *mut cef_print_handler_t,
    device_units_per_inch: c_int,
) -> cef_size_t {
    let mm_per_in = 25.4;
    // TODO: less hacky / hard-coded?
    let device_units_per_mm = (device_units_per_inch as f64) / mm_per_in;
    let width = 210.0 * device_units_per_mm;
    let height = 297.0 * device_units_per_mm;
    cef_size_t {
        width: width as i32,
        height: height as i32,
    }
}

pub fn allocate() -> *mut PrintHandler {
    let handler = PrintHandler {
        print_handler: cef_print_handler_t {
            base: cef_base_ref_counted_t {
                size: size_of::<PrintHandler>() as u64,
                add_ref: Some(add_ref),
                release: Some(release),
                has_one_ref: Some(has_one_ref),
                has_at_least_one_ref: Some(has_at_least_one_ref),
            },
            // TODO: implement these?
            on_print_start: None,
            on_print_settings: None,
            on_print_dialog: None,
            on_print_job: None,
            on_print_reset: None,
            get_pdf_paper_size: Some(get_pdf_paper_size),
        },
        ref_count: AtomicUsize::new(1),
    };

    Box::into_raw(Box::from(handler))
}

extern "C" fn add_ref(base: *mut cef_base_ref_counted_t) {
    let print_handler = base as *mut PrintHandler;
    unsafe {
        (*print_handler).ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn release(base: *mut cef_base_ref_counted_t) -> c_int {
    let print_handler = base as *mut PrintHandler;
    let count = unsafe { (*print_handler).ref_count.fetch_sub(1, Ordering::SeqCst) - 1 };

    if count == 0 {
        unsafe {
            Box::from_raw(print_handler);
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let print_handler = base as *mut PrintHandler;
    let count = unsafe { (*print_handler).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let print_handler = base as *mut PrintHandler;
    let count = unsafe { (*print_handler).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
