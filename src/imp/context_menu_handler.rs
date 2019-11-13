use libc::{calloc, free};
use std::mem::size_of;
use std::os::raw::{c_int, c_void};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{
    cef_base_ref_counted_t, cef_browser_t, cef_context_menu_handler_t, cef_context_menu_params_t,
    cef_frame_t, cef_menu_model_t,
};

#[derive(Debug)]
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
        //(*model).remove.unwrap()(model, super::bindings::cef_menu_id_t_MENU_ID_VIEW_SOURCE as i32);
    }
}

pub fn allocate() -> *mut ContextMenuHandler {
    let context_menu_handler =
        unsafe { calloc(1, size_of::<ContextMenuHandler>()) as *mut ContextMenuHandler };
    unsafe {
        (*context_menu_handler).context_menu_handler.base.size = size_of::<ContextMenuHandler>();
        (*context_menu_handler).ref_count.store(1, Ordering::SeqCst);
        (*context_menu_handler).context_menu_handler.base.add_ref = Some(add_ref);
        (*context_menu_handler).context_menu_handler.base.release = Some(release);
        (*context_menu_handler)
            .context_menu_handler
            .base
            .has_one_ref = Some(has_one_ref);
        (*context_menu_handler)
            .context_menu_handler
            .base
            .has_at_least_one_ref = Some(has_at_least_one_ref);

        (*context_menu_handler)
            .context_menu_handler
            .on_before_context_menu = Some(on_before_context_menu);
    };

    context_menu_handler
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
            free(context_menu_handler as *mut c_void);
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
