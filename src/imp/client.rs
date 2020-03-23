use std::mem::size_of;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{
    cef_base_ref_counted_t, cef_browser_t, cef_client_t, cef_context_menu_handler_t,
    cef_display_handler_t, cef_frame_t, cef_life_span_handler_t, cef_process_id_t,
    cef_process_message_t, cef_request_handler_t, cef_string_t, cef_string_userfree_t,
    cef_string_userfree_utf16_free, cef_window_t,
};
use super::context_menu_handler::{self, ContextMenuHandler};
use super::display_handler::{self, DisplayHandler};
use super::life_span_handler::{self, LifeSpanHandler};
use super::request_handler::{self, RequestHandler};

#[repr(C)]
pub struct Client {
    client: cef_client_t,
    ref_count: AtomicUsize,
    life_span_handler: *mut LifeSpanHandler,
    context_menu_handler: *mut ContextMenuHandler,
    request_handler: *mut RequestHandler,
    display_handler: *mut DisplayHandler,
}

impl Client {
    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn get_life_span_handler(slf: *mut cef_client_t) -> *mut cef_life_span_handler_t {
    let client = slf as *mut Client;
    let handler = unsafe { (*client).life_span_handler };
    unsafe { (*handler).inc_ref() };
    handler as *mut cef_life_span_handler_t
}

extern "C" fn get_context_menu_handler(slf: *mut cef_client_t) -> *mut cef_context_menu_handler_t {
    let client = slf as *mut Client;
    let handler = unsafe { (*client).context_menu_handler };
    unsafe { (*handler).inc_ref() };
    handler as *mut cef_context_menu_handler_t
}

extern "C" fn get_request_handler(slf: *mut cef_client_t) -> *mut cef_request_handler_t {
    let client = slf as *mut Client;
    let handler = unsafe { (*client).request_handler };
    unsafe { (*handler).inc_ref() };
    handler as *mut cef_request_handler_t
}

extern "C" fn get_display_handler(slf: *mut cef_client_t) -> *mut cef_display_handler_t {
    let client = slf as *mut Client;
    let handler = unsafe { (*client).display_handler };
    unsafe { (*handler).inc_ref() };
    handler as *mut cef_display_handler_t
}

unsafe extern "C" fn on_process_message_received(
    _slf: *mut cef_client_t,
    browser: *mut cef_browser_t,
    frame: *mut cef_frame_t,
    _source_process: cef_process_id_t,
    message: *mut cef_process_message_t,
) -> c_int {
    let cef_message_name: cef_string_userfree_t =
        ((*message).get_name.expect("get_name is a function"))(message);
    let chars: *mut u16 = (*cef_message_name).str;
    let len: usize = (*cef_message_name).length as usize;
    let chars = std::slice::from_raw_parts(chars, len);
    let message_name = std::char::decode_utf16(chars.iter().cloned())
        .map(|r| r.unwrap_or(std::char::REPLACEMENT_CHARACTER))
        .collect::<String>();
    cef_string_userfree_utf16_free(cef_message_name);

    log::debug!("browser process recieved `{}` message", message_name);
    if message_name == "print_to_pdf" {
        // get the path
        let args = ((*message)
            .get_argument_list
            .expect("get_argument_list is a function"))(message);
        let cef_path: cef_string_userfree_t =
            ((*args).get_string.expect("get_string is a function"))(args, 0);
        let chars: *mut u16 = (*cef_path).str;
        let len: usize = (*cef_path).length as usize;
        let chars = std::slice::from_raw_parts(chars, len);
        let path = std::char::decode_utf16(chars.iter().cloned())
            .map(|r| r.unwrap_or(std::char::REPLACEMENT_CHARACTER))
            .collect::<String>();
        cef_string_userfree_utf16_free(cef_path);

        super::browser::print_to_pdf(
            browser,
            path,
            Some(Box::from(move |ok| {
                // now send an IPC message back to the renderer
                // convert the message name to a CEF string
                let mut cef_message_name = cef_string_t::default();
                let message_name = "print_to_pdf_done".as_bytes();
                let message_name = std::ffi::CString::new(message_name).unwrap();
                super::bindings::cef_string_utf8_to_utf16(
                    message_name.as_ptr(),
                    message_name.to_bytes().len() as u64,
                    &mut cef_message_name,
                );

                // build the message
                let message = super::bindings::cef_process_message_create(&cef_message_name);
                let args = ((*message)
                    .get_argument_list
                    .expect("get_argument_list is a function"))(message);
                ((*args).set_size.expect("set_size is a function"))(args, 1);
                ((*args).set_bool.expect("set_bool is a function"))(args, 0, ok as i32);

                // send the message
                ((*frame)
                    .send_process_message
                    .expect("send_process_message is a function"))(
                    frame,
                    super::bindings::cef_process_id_t_PID_RENDERER,
                    message,
                );
            })),
        );

        1
    } else if message_name == "save_file_dialog" || message_name == "open_file_dialog" {
        let args = ((*message)
            .get_argument_list
            .expect("get_argument_list is a function"))(message);

        let num_args = (*args).get_size.expect("get_size is a function")(args);
        debug_assert_eq!(num_args, 3);

        // get the title
        let cef_title: cef_string_userfree_t =
            ((*args).get_string.expect("get_string is a function"))(args, 0);
        let title: String = if cef_title == std::ptr::null_mut() {
            let chars: *mut u16 = (*cef_title).str;
            let len: usize = (*cef_title).length as usize;
            let chars = std::slice::from_raw_parts(chars, len);
            let title = std::char::decode_utf16(chars.iter().cloned())
                .map(|r| r.unwrap_or(std::char::REPLACEMENT_CHARACTER))
                .collect::<String>();
            cef_string_userfree_utf16_free(cef_title);
            title
        }
        else {
            "".to_owned()
        };

        // get the initial_file_name
        let cef_initial_file_name: cef_string_userfree_t =
            ((*args).get_string.expect("get_string is a function"))(args, 1);
        let initial_file_name: String = if cef_initial_file_name == std::ptr::null_mut() {
            "".to_owned()
        }
        else {
            let chars: *mut u16 = (*cef_initial_file_name).str;
            let len: usize = (*cef_initial_file_name).length as usize;
            let chars = std::slice::from_raw_parts(chars, len);
            let initial_file_name = std::char::decode_utf16(chars.iter().cloned())
                .map(|r| r.unwrap_or(std::char::REPLACEMENT_CHARACTER))
                .collect::<String>();
            cef_string_userfree_utf16_free(cef_initial_file_name);
            initial_file_name
        };

        // get the filter
        let cef_filter: cef_string_userfree_t =
            ((*args).get_string.expect("get_string is a function"))(args, 2);
        let filter: String = if cef_filter == std::ptr::null_mut() {
            let chars: *mut u16 = (*cef_filter).str;
            let len: usize = (*cef_filter).length as usize;
            let chars = std::slice::from_raw_parts(chars, len);
            let filter = std::char::decode_utf16(chars.iter().cloned())
                .map(|r| r.unwrap_or(std::char::REPLACEMENT_CHARACTER))
                .collect::<String>();
            cef_string_userfree_utf16_free(cef_filter);
            filter
        }
        else {
            "".to_owned()
        };

        super::browser::run_file_dialog(
            browser,
            match message_name.as_ref() {
                "open_file_dialog" => super::v8_file_dialog_handler::FileDialogMode::Open,
                "save_file_dialog" => super::v8_file_dialog_handler::FileDialogMode::Save,
                _ => unreachable!()
            },
            title,
            initial_file_name,
            filter,
            Some(Box::from(move |path: Option<std::path::PathBuf>| {
                log::debug!("client save callback");
                // now send an IPC message back to the renderer
                // convert the message name to a CEF string
                let mut cef_message_name = cef_string_t::default();
                let message_name = "run_file_dialog_done".as_bytes();
                let message_name = std::ffi::CString::new(message_name).unwrap();
                super::bindings::cef_string_utf8_to_utf16(
                    message_name.as_ptr(),
                    message_name.to_bytes().len() as u64,
                    &mut cef_message_name,
                );

                // build the message
                let message = super::bindings::cef_process_message_create(&cef_message_name);
                let args = ((*message)
                    .get_argument_list
                    .expect("get_argument_list is a function"))(message);
                if let Some(path) = path {
                    ((*args).set_size.expect("set_size is a function"))(args, 1);

                    let mut cef_path = cef_string_t::default();
                    let path = path.display().to_string();
                    let path = path.as_bytes();
                    let path = std::ffi::CString::new(path).unwrap();
                    super::bindings::cef_string_utf8_to_utf16(
                        path.as_ptr(),
                        path.to_bytes().len() as u64,
                        &mut cef_path,
                    );
                    ((*args).set_string.expect("set_string is a function"))(args, 0, &cef_path);
                } else {
                    ((*args).set_size.expect("set_size is a function"))(args, 0);
                }

                // and finally send the message
                ((*frame)
                    .send_process_message
                    .expect("send_process_message is a function"))(
                    frame,
                    super::bindings::cef_process_id_t_PID_RENDERER,
                    message,
                );
            })),
        );

        1
    } else {
        0
    }
}

pub fn allocate(window: *mut cef_window_t) -> *mut Client {
    let client = Client {
        client: cef_client_t {
            base: cef_base_ref_counted_t {
                size: size_of::<Client>() as u64,
                add_ref: Some(add_ref),
                release: Some(release),
                has_one_ref: Some(has_one_ref),
                has_at_least_one_ref: Some(has_at_least_one_ref),
            },
            get_context_menu_handler: Some(get_context_menu_handler),
            get_dialog_handler: None,
            get_display_handler: Some(get_display_handler),
            get_download_handler: None,
            get_drag_handler: None,
            get_find_handler: None,
            get_focus_handler: None,
            get_jsdialog_handler: None,
            get_keyboard_handler: None,
            get_life_span_handler: Some(get_life_span_handler),
            get_load_handler: None,
            get_render_handler: None,
            get_request_handler: Some(get_request_handler),
            on_process_message_received: Some(on_process_message_received),
        },
        ref_count: AtomicUsize::new(1),
        life_span_handler: life_span_handler::allocate(),
        context_menu_handler: context_menu_handler::allocate(),
        request_handler: request_handler::allocate(),
        display_handler: display_handler::allocate(window),
    };

    Box::into_raw(Box::from(client))
}

extern "C" fn add_ref(base: *mut cef_base_ref_counted_t) {
    let client = base as *mut Client;
    unsafe {
        (*client).ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn release(base: *mut cef_base_ref_counted_t) -> c_int {
    let client = base as *mut Client;
    let count = unsafe { (*client).ref_count.fetch_sub(1, Ordering::SeqCst) - 1 };

    if count == 0 {
        unsafe {
            Box::from_raw(client);
            // TODO: free our handlers here too?
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let client = base as *mut Client;
    let count = unsafe { (*client).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let client = base as *mut Client;
    let count = unsafe { (*client).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
