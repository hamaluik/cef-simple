use std::mem::size_of;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicUsize, Ordering};

use cef_simple_sys::{
    cef_base_ref_counted_t, cef_browser_t, cef_context_menu_handler_t, cef_context_menu_params_t,
    cef_frame_t, cef_menu_model_t,
};

#[repr(C)]
pub struct ContextMenuHandler {
    context_menu_handler: cef_context_menu_handler_t,
    ref_count: AtomicUsize,
}

impl ContextMenuHandler {
    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn on_before_context_menu(
    _slf: *mut cef_context_menu_handler_t,
    _browser: *mut cef_browser_t,
    _frame: *mut cef_frame_t,
    _params: *mut cef_context_menu_params_t,
    model: *mut cef_menu_model_t,
) {
    unsafe {
        (*model).clear.unwrap()(model);
        //(*model).remove.unwrap()(model, cef_simple_sys::cef_menu_id_t_MENU_ID_VIEW_SOURCE as i32);
    }
}

pub fn allocate() -> *mut ContextMenuHandler {
    let handler = ContextMenuHandler {
        context_menu_handler: cef_context_menu_handler_t {
            base: cef_base_ref_counted_t {
                size: size_of::<ContextMenuHandler>() as u64,
                add_ref: Some(add_ref),
                release: Some(release),
                has_one_ref: Some(has_one_ref),
                has_at_least_one_ref: Some(has_at_least_one_ref),
            },
            on_before_context_menu: Some(on_before_context_menu),
            run_context_menu: None,
            on_context_menu_command: None,
            on_context_menu_dismissed: None,
        },
        ref_count: AtomicUsize::new(1),
    };

    Box::into_raw(Box::from(handler))
}

extern "C" fn add_ref(base: *mut cef_base_ref_counted_t) {
    let context_menu_handler = base as *mut ContextMenuHandler;
    unsafe {
        (*context_menu_handler)
            .ref_count
            .fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn release(base: *mut cef_base_ref_counted_t) -> c_int {
    let context_menu_handler = base as *mut ContextMenuHandler;
    let count = unsafe {
        (*context_menu_handler)
            .ref_count
            .fetch_sub(1, Ordering::SeqCst)
            - 1
    };

    if count == 0 {
        unsafe {
            Box::from_raw(context_menu_handler);
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let context_menu_handler = base as *mut ContextMenuHandler;
    let count = unsafe { (*context_menu_handler).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let context_menu_handler = base as *mut ContextMenuHandler;
    let count = unsafe { (*context_menu_handler).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
