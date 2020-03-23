use std::mem::size_of;
use std::os::raw::{c_int};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{
    cef_base_ref_counted_t, cef_v8handler_t, cef_string_t, cef_v8value_t, size_t,
    cef_string_userfree_t, cef_string_userfree_utf16_free, cef_frame_t, cef_v8context_t,
    cef_v8context_get_current_context, cef_process_message_t, cef_v8value_create_string
};

pub enum FileDialogMode {
    Open,
    Save,
}

#[repr(C)]
pub struct V8FileDialogHandler {
    v8_handler: cef_v8handler_t,
    ref_count: AtomicUsize,
    pub frame: Option<*mut cef_frame_t>,
    pub done_callback: Option<(*mut cef_v8context_t, *mut cef_v8value_t, *mut cef_v8value_t)>,
}

const CODE: &str = r#"
    var cef;
    if(!cef) cef = {};
    (function() {
        cef.saveFileDialog = function(title, defaultFileName, filter) {
            native function saveFileDialog(title, defaultFileName, filter, onDone, onError);
            return new Promise((resolve, reject) => {
                saveFileDialog(title, defaultFileName, filter, resolve, reject);
            });
        };
        cef.openFileDialog = function(title, defaultFileName, filter) {
            native function openFileDialog(title, defaultFileName, filter, onDone, onError);
            return new Promise((resolve, reject) => {
                openFileDialog(title, defaultFileName, filter, resolve, reject);
            });
        };
    })();
"#;

pub unsafe fn register_extension(extension: *mut V8FileDialogHandler) {
    use std::ffi::CString;
    use super::bindings::{cef_string_utf8_to_utf16, cef_register_extension};
    let code = CODE.as_bytes();
    let code = CString::new(code).unwrap();
    let mut cef_code = cef_string_t::default();
    cef_string_utf8_to_utf16(code.as_ptr(), code.to_bytes().len() as u64, &mut cef_code);

    let extension_name = "CEF File Dialogs";
    let extension_name = extension_name.as_bytes();
    let extension_name = CString::new(extension_name).unwrap();
    let mut cef_extension_name = cef_string_t::default();
    cef_string_utf8_to_utf16(extension_name.as_ptr(), extension_name.to_bytes().len() as u64, &mut cef_extension_name);

    cef_register_extension(&cef_extension_name, &cef_code, extension as *mut cef_v8handler_t);
    log::debug!("registered file dialogs extension");
}

pub unsafe fn process_message(slf: *mut V8FileDialogHandler, message_name: &str, message: *mut cef_process_message_t) -> bool {
    if message_name != "run_file_dialog_done" {
        return false;
    }

    let args = ((*message).get_argument_list.expect("get_argument_list is a function"))(message);
    let size = (*args).get_size.expect("get_size is a function")(args);
    if size < 1 {
        on_file_dialog_done(slf, None);
    }
    else {
        let cef_path: cef_string_userfree_t = (*args).get_string.expect("get_string is a function")(args, 0);
        on_file_dialog_done(slf, Some(cef_path));
        cef_string_userfree_utf16_free(cef_path);
    }

    true
}

unsafe fn on_file_dialog_done(slf: *mut V8FileDialogHandler, path: Option<*const cef_string_t>) {
    if let Some((context, on_success, on_error)) = (*slf).done_callback {
        ((*context).enter.expect("enter is a function"))(context);

        if let Some(path) = path {
            // store the path as a string in v8
            let v8_path = cef_v8value_create_string(path);
            // and call success, with a single argument that is the path!
            ((*on_success).execute_function.expect("execute_function is a function"))(on_success, std::ptr::null_mut(), 1, &v8_path);
        }
        else {
            // if the path was None, the user cancelled; report that as an error to JS
            ((*on_error).execute_function.expect("execute_function is a function"))(on_error, std::ptr::null_mut(), 0, std::ptr::null_mut());
        }

        ((*context).exit.expect("exit is a function"))(context);
        (*slf).done_callback = None;
    }
    else {
        log::warn!("file dialog is done but callback wasn't set?!");
    }
}

unsafe extern "C" fn execute(
    slf: *mut cef_v8handler_t,
    name: *const cef_string_t,
    _object: *mut cef_v8value_t,
    arguments_count: size_t,
    arguments: *const *mut cef_v8value_t,
    _retval: *mut *mut cef_v8value_t,
    _exception: *mut cef_string_t,
) -> c_int {
    // get the name of the function
    let chars: *mut u16 = (*name).str;
    let len: usize = (*name).length as usize;
    let chars = std::slice::from_raw_parts(chars, len);
    let name = std::char::decode_utf16(chars.iter().cloned())
        .map(|r| r.unwrap_or(std::char::REPLACEMENT_CHARACTER))
        .collect::<String>();

    if (name == "saveFileDialog" || name == "openFileDialog") && arguments_count == 5 {
        // get the title argument
        let arg_title: *mut cef_v8value_t = *arguments.offset(0);
        let is_string = ((*arg_title).is_string.expect("is_string is a function"))(arg_title) == 1;
        if !is_string {
            log::warn!("title argument isn't a string!");
            return 0;
        }

        // get the file name argument
        let arg_file_name: *mut cef_v8value_t = *arguments.offset(1);
        let is_string: bool = ((*arg_file_name).is_string.expect("is_string is a function"))(arg_file_name) == 1;
        let arg_file_name_is_null: bool = ((*arg_file_name).is_null.expect("is_null is a function"))(arg_file_name) == 1;
        if !arg_file_name_is_null && !is_string {
            log::error!("file name argument isn't a string, nor is it null!");
            return 0;
        }

        // get the filter argument
        let arg_filter: *mut cef_v8value_t = *arguments.offset(2);
        let is_string = ((*arg_filter).is_string.expect("is_string is a function"))(arg_filter) == 1;
        if !is_string {
            log::warn!("filter argument isn't a string!");
            return 0;
        }

        // get the onDone argument
        let arg_on_done: *mut cef_v8value_t = *arguments.offset(3);
        let is_function = ((*arg_on_done).is_function.expect("is_function is a function"))(arg_on_done) == 1;
        if !is_function {
            log::warn!("onDone argument isn't a function!");
            return 0;
        }

        // get the onError argument
        let arg_on_error: *mut cef_v8value_t = *arguments.offset(4);
        let is_function = ((*arg_on_error).is_function.expect("is_function is a function"))(arg_on_error) == 1;
        if !is_function {
            log::warn!("onError argument isn't a function!");
            return 0;
        }

        // get the v8 strings as cef strings
        let cef_title: cef_string_userfree_t = ((*arg_title).get_string_value.expect("get_string_value is a function"))(arg_title);
        let cef_file_name: cef_string_userfree_t = if arg_file_name_is_null {
            super::bindings::cef_string_userfree_utf16_alloc()
        }
        else {
            ((*arg_file_name).get_string_value.expect("get_string_value is a function"))(arg_file_name)
        };
        let cef_filter: cef_string_userfree_t = ((*arg_filter).get_string_value.expect("get_string_value is a function"))(arg_filter);

        // now send an IPC message to the frame process telling it to print
        let _self = slf as *mut V8FileDialogHandler;
        if let Some(frame) = (*_self).frame {
            // convert the message name to a CEF string
            let mut cef_message_name = cef_string_t::default();
            let message_name = match name.as_ref() {
                "openFileDialog" => "open_file_dialog".as_bytes(),
                "saveFileDialog" => "save_file_dialog".as_bytes(),
                _ => unreachable!(),
            };
            let message_name = std::ffi::CString::new(message_name).unwrap();
            super::bindings::cef_string_utf8_to_utf16(message_name.as_ptr(), message_name.to_bytes().len() as u64, &mut cef_message_name);

            // store our callback to onDone
            let context = cef_v8context_get_current_context();
            (*_self).done_callback = Some((context, arg_on_done, arg_on_error));

            // build the message
            let message = super::bindings::cef_process_message_create(&cef_message_name);
            let args = ((*message).get_argument_list.expect("get_argument_list is a function"))(message);
            ((*args).set_size.expect("set_size is a function"))(args, 3);
            ((*args).set_string.expect("set_string is a function"))(args, 0, cef_title);
            ((*args).set_string.expect("set_string is a function"))(args, 1, cef_file_name);
            ((*args).set_string.expect("set_string is a function"))(args, 2, cef_filter);

            // send the message
            ((*frame).send_process_message.expect("send_process_message is a function"))(frame, super::bindings::cef_process_id_t_PID_BROWSER, message);
        }
        else {
            log::error!("frame isn't set!");
        }

        cef_string_userfree_utf16_free(cef_title);
        cef_string_userfree_utf16_free(cef_file_name);
        cef_string_userfree_utf16_free(cef_filter);
        1
    }
    else {
        log::warn!("unrecognized function: `{}` with {} args, skipping", name, arguments_count);
        0
    }
}

pub fn allocate() -> *mut V8FileDialogHandler {
    let handler = V8FileDialogHandler {
        v8_handler: cef_v8handler_t {
            base: cef_base_ref_counted_t {
                size: size_of::<V8FileDialogHandler>() as u64,
                add_ref: Some(add_ref),
                release: Some(release),
                has_one_ref: Some(has_one_ref),
                has_at_least_one_ref: Some(has_at_least_one_ref),
            },
            execute: Some(execute),
        },
        ref_count: AtomicUsize::new(1),
        frame: None,
        done_callback: None,
    };

    Box::into_raw(Box::from(handler))
}

extern "C" fn add_ref(base: *mut cef_base_ref_counted_t) {
    let v8_handler = base as *mut V8FileDialogHandler;
    unsafe { (*v8_handler).ref_count.fetch_add(1, Ordering::SeqCst) };
}

extern "C" fn release(base: *mut cef_base_ref_counted_t) -> c_int {
    let v8_handler = base as *mut V8FileDialogHandler;
    let count = unsafe { (*v8_handler).ref_count.fetch_sub(1, Ordering::SeqCst) - 1 };

    if count == 0 {
        unsafe {
            Box::from_raw(v8_handler);
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let v8_handler = base as *mut V8FileDialogHandler;
    let count = unsafe { (*v8_handler).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let v8_handler = base as *mut V8FileDialogHandler;
    let count = unsafe { (*v8_handler).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
