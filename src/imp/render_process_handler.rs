use std::mem::size_of;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{
    cef_base_ref_counted_t, cef_browser_t, cef_dictionary_value_t, cef_frame_t, cef_process_id_t,
    cef_process_message_t, cef_render_process_handler_t, cef_string_userfree_t,
    cef_string_userfree_utf16_free,
};
use super::v8_file_dialog_handler::{self, V8FileDialogHandler};
use super::v8_pdf_print_handler::{self, V8PDFPrintHandler};

#[repr(C)]
pub struct RenderProcessHandler {
    render_process_handler: cef_render_process_handler_t,
    ref_count: AtomicUsize,
    pdf_print_extension: *mut V8PDFPrintHandler,
    file_dialog_extension: *mut V8FileDialogHandler,
}

impl RenderProcessHandler {
    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

unsafe extern "C" fn on_web_kit_initialized(slf: *mut cef_render_process_handler_t) {
    let _self = slf as *mut RenderProcessHandler;
    super::v8_pdf_print_handler::register_extension((*_self).pdf_print_extension);
    super::v8_file_dialog_handler::register_extension((*_self).file_dialog_extension);
    log::debug!("web kit initialized");
}

unsafe extern "C" fn on_browser_created(
    slf: *mut cef_render_process_handler_t,
    browser: *mut cef_browser_t,
    _extra_info: *mut cef_dictionary_value_t,
) {
    log::debug!("browser created");
    let _self = slf as *mut RenderProcessHandler;
    (*(*_self).pdf_print_extension).browser = Some(browser);
    (*(*_self).file_dialog_extension).browser = Some(browser);
}

unsafe extern "C" fn on_browser_destroyed(
    slf: *mut cef_render_process_handler_t,
    _browser: *mut cef_browser_t,
) {
    log::debug!("browser destroyed");
    let _self = slf as *mut RenderProcessHandler;
    (*(*_self).pdf_print_extension).browser = None;
    (*(*_self).file_dialog_extension).browser = None;
}

unsafe extern "C" fn on_process_message_received(
    slf: *mut cef_render_process_handler_t,
    _browser: *mut cef_browser_t,
    _frame: *mut cef_frame_t,
    _source_process: cef_process_id_t,
    message: *mut cef_process_message_t,
) -> c_int {
    let cef_message_name: cef_string_userfree_t =
        ((*message).get_name.expect("get_name is a function"))(message);
    let chars: *mut u16 = (*cef_message_name).str_;
    let len: usize = (*cef_message_name).length as usize;
    let chars = std::slice::from_raw_parts(chars, len);
    let message_name = std::char::decode_utf16(chars.iter().cloned())
        .map(|r| r.unwrap_or(std::char::REPLACEMENT_CHARACTER))
        .collect::<String>();
    cef_string_userfree_utf16_free(cef_message_name);
    log::debug!("renderer received message: {}", message_name);

    let _self = slf as *mut RenderProcessHandler;
    if super::v8_pdf_print_handler::process_message(
        (*_self).pdf_print_extension,
        &message_name,
        message,
    ) {
        return 1;
    }
    if super::v8_file_dialog_handler::process_message(
        (*_self).file_dialog_extension,
        &message_name,
        message,
    ) {
        return 1;
    }
    log::warn!("unhandled process message in renderer: `{}`", message_name);
    0
}

pub fn allocate() -> *mut RenderProcessHandler {
    let handler = RenderProcessHandler {
        render_process_handler: cef_render_process_handler_t {
            base: cef_base_ref_counted_t {
                size: size_of::<RenderProcessHandler>() as u64,
                add_ref: Some(add_ref),
                release: Some(release),
                has_one_ref: Some(has_one_ref),
                has_at_least_one_ref: Some(has_at_least_one_ref),
            },
            //on_render_thread_created: None,
            on_web_kit_initialized: Some(on_web_kit_initialized),
            on_browser_created: Some(on_browser_created),
            on_browser_destroyed: Some(on_browser_destroyed),
            get_load_handler: None,
            on_context_created: None,
            on_context_released: None,
            on_uncaught_exception: None,
            on_focused_node_changed: None,
            on_process_message_received: Some(on_process_message_received),
        },
        ref_count: AtomicUsize::new(1),
        pdf_print_extension: v8_pdf_print_handler::allocate(),
        file_dialog_extension: v8_file_dialog_handler::allocate(),
    };

    Box::into_raw(Box::from(handler))
}

extern "C" fn add_ref(base: *mut cef_base_ref_counted_t) {
    let render_process_handler = base as *mut RenderProcessHandler;
    unsafe {
        (*render_process_handler)
            .ref_count
            .fetch_add(1, Ordering::SeqCst)
    };
}

extern "C" fn release(base: *mut cef_base_ref_counted_t) -> c_int {
    let render_process_handler = base as *mut RenderProcessHandler;
    let count = unsafe {
        (*render_process_handler)
            .ref_count
            .fetch_sub(1, Ordering::SeqCst)
            - 1
    };

    if count == 0 {
        unsafe {
            Box::from_raw(render_process_handler);
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let render_process_handler = base as *mut RenderProcessHandler;
    let count = unsafe { (*render_process_handler).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let render_process_handler = base as *mut RenderProcessHandler;
    let count = unsafe { (*render_process_handler).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
