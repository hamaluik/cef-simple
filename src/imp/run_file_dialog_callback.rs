use std::mem::size_of;
use std::os::raw::{c_int};
use std::sync::atomic::{AtomicUsize, Ordering};
use super::bindings::{
    cef_base_ref_counted_t, _cef_run_file_dialog_callback_t, cef_string_list_t,
    cef_string_list_value, cef_string_list_size, cef_string_userfree_utf16_alloc,
    cef_string_userfree_t, cef_string_userfree_utf16_free,
};

#[repr(C)]
pub struct RunFileDialogCallback {
    run_file_dialog_callback: _cef_run_file_dialog_callback_t,
    ref_count: AtomicUsize,
    on_done: Option<Box<dyn FnMut(Option<std::path::PathBuf>)>>,
}

unsafe extern "C" fn on_file_dialog_dismissed(slf: *mut _cef_run_file_dialog_callback_t, _selected_accept_filter: c_int, file_paths: cef_string_list_t) {
    let callback = slf as *mut RunFileDialogCallback;
    if let Some(on_done) = &mut (*callback).on_done {
        // if they cancelled, file_paths will be null, so alert as much
        if file_paths == std::ptr::null_mut() || cef_string_list_size(file_paths) < 1 {
            log::debug!("user cancelled file dialog");
            on_done(None);
        }
        else {
            // extract the first string from the list (only support a single string for now)
            let cef_path: cef_string_userfree_t = cef_string_userfree_utf16_alloc();
            if cef_string_list_value(file_paths, 0, cef_path) != 1 {
                log::warn!("failed to extract first path from file dialog callback");
                on_done(None);
                return;
            }

            // cover the path into a Rust string
            let chars: *mut u16 = (*cef_path).str;
            let len: usize = (*cef_path).length as usize;
            let chars = std::slice::from_raw_parts(chars, len);
            let path = std::char::decode_utf16(chars.iter().cloned())
                .map(|r| r.unwrap_or(std::char::REPLACEMENT_CHARACTER))
                .collect::<String>();
            cef_string_userfree_utf16_free(cef_path);

            // and alert our listener
            on_done(Some(std::path::PathBuf::from(path)));
        }
    }
    else {
        log::warn!("no callback registered for run file dialog callback, is this intentional?");
    }
}

pub fn allocate(on_done: Option<Box<dyn FnMut(Option<std::path::PathBuf>)>>) -> *mut RunFileDialogCallback {
    let handler = RunFileDialogCallback {
        run_file_dialog_callback: _cef_run_file_dialog_callback_t {
            base: cef_base_ref_counted_t {
                size: size_of::<RunFileDialogCallback>() as u64,
                add_ref: Some(add_ref_run_file_dialog_callback),
                release: Some(release_run_file_dialog_callback),
                has_one_ref: Some(has_one_ref_run_file_dialog_callback),
                has_at_least_one_ref: Some(has_at_least_one_ref_run_file_dialog_callback),
            },
            on_file_dialog_dismissed: Some(on_file_dialog_dismissed),
        },
        ref_count: AtomicUsize::new(1),
        on_done,
    };

    Box::into_raw(Box::from(handler))
}

extern "C" fn add_ref_run_file_dialog_callback(base: *mut cef_base_ref_counted_t) {
    let life_span_handler = base as *mut RunFileDialogCallback;
    unsafe {
        (*life_span_handler)
            .ref_count
            .fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn release_run_file_dialog_callback(base: *mut cef_base_ref_counted_t) -> c_int {
    let life_span_handler = base as *mut RunFileDialogCallback;
    let count = unsafe {
        (*life_span_handler)
            .ref_count
            .fetch_sub(1, Ordering::SeqCst)
            - 1
    };

    if count == 0 {
        unsafe {
            Box::from_raw(life_span_handler);
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref_run_file_dialog_callback(base: *mut cef_base_ref_counted_t) -> c_int {
    let life_span_handler = base as *mut RunFileDialogCallback;
    let count = unsafe { (*life_span_handler).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref_run_file_dialog_callback(base: *mut cef_base_ref_counted_t) -> c_int {
    let life_span_handler = base as *mut RunFileDialogCallback;
    let count = unsafe { (*life_span_handler).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
