use super::bindings::{
    cef_browser_t, cef_string_list_alloc, cef_string_list_append, cef_string_t,
    cef_string_utf8_to_utf16,
};
use super::print_pdf_callback;
use super::run_file_dialog_callback;
use std::ffi::CString;

pub unsafe fn print_to_pdf<P: AsRef<std::path::Path>>(
    browser: *mut cef_browser_t,
    path: P,
    on_done: Option<Box<dyn FnMut(bool)>>,
) {
    log::debug!("printing PDF to path `{}`...", path.as_ref().display());

    // get our browser host
    let host = (*browser).get_host.unwrap()(browser);

    // first, convert the path to a cef string
    let path: String = path.as_ref().display().to_string();
    let path = path.as_bytes();
    let path = CString::new(path).unwrap();
    let mut cef_path = cef_string_t::default();
    cef_string_utf8_to_utf16(path.as_ptr(), path.to_bytes().len() as u64, &mut cef_path);

    // determine the settings
    // note: page size in microns, to get microns from inches, multiply
    // by 25400.
    // TODO: different paper sizes?
    let settings = super::bindings::_cef_pdf_print_settings_t {
        header_footer_title: cef_string_t::default(), // empty header / footer
        header_footer_url: cef_string_t::default(),   // empty url
        page_width: 210000,                           // 210 mm (a4 paper)
        page_height: 297000,                          // 297 mm (a4 paper)
        scale_factor: 100,                            // scale the page at 100% (i.e. don't.)
        margin_top: 0, // margins in millimeters (actually ignored because of margin type)
        margin_right: 0,
        margin_bottom: 0,
        margin_left: 0,
        margin_type: super::bindings::cef_pdf_print_margin_type_t_PDF_PRINT_MARGIN_DEFAULT, // default margins as defined by chrome, ~1 inch
        header_footer_enabled: 0, // no headers or footers
        selection_only: 0,        // print everything
        landscape: 0,             // portrait mode
        backgrounds_enabled: 1,   // show background colours / graphics
    };

    // now a callback when the print is done
    let callback = print_pdf_callback::allocate(on_done);

    // finally, initiate the print
    (*host).print_to_pdf.expect("print_to_pdf is a function")(
        host,
        &mut cef_path,
        &settings,
        callback as *mut super::bindings::_cef_pdf_print_callback_t,
    );
}

pub unsafe fn run_file_dialog(
    browser: *mut cef_browser_t,
    mode: super::v8_file_dialog_handler::FileDialogMode,
    title: String,
    initial_file_name: String,
    filter: String,
    on_done: Option<Box<dyn FnMut(Option<std::path::PathBuf>)>>,
) {
    log::debug!("launching file dialog in mode `{:?}`...", mode);

    // get our browser host
    let host = (*browser).get_host.unwrap()(browser);

    // convert the title to a cef string
    let title = title.as_bytes();
    let title = CString::new(title).unwrap();
    let mut cef_title = cef_string_t::default();
    cef_string_utf8_to_utf16(
        title.as_ptr(),
        title.to_bytes().len() as u64,
        &mut cef_title,
    );

    // convert the initial_file_name to a cef string
    let initial_file_name = initial_file_name.as_bytes();
    let initial_file_name = CString::new(initial_file_name).unwrap();
    let mut cef_initial_file_name = cef_string_t::default();
    cef_string_utf8_to_utf16(
        initial_file_name.as_ptr(),
        initial_file_name.to_bytes().len() as u64,
        &mut cef_initial_file_name,
    );

    // convert the filter to a cef string
    let filter = filter.as_bytes();
    let filter = CString::new(filter).unwrap();
    let mut cef_filter = cef_string_t::default();
    cef_string_utf8_to_utf16(
        filter.as_ptr(),
        filter.to_bytes().len() as u64,
        &mut cef_filter,
    );

    // build the filter list
    let filters = cef_string_list_alloc();
    cef_string_list_append(filters, &cef_filter);

    // and a callback
    let callback = run_file_dialog_callback::allocate(on_done);

    // and run the dialog
    (*host)
        .run_file_dialog
        .expect("run_file_dialog is a function")(
        host,
        match mode {
            super::v8_file_dialog_handler::FileDialogMode::Open => {
                super::bindings::cef_file_dialog_mode_t_FILE_DIALOG_OPEN
            }
            super::v8_file_dialog_handler::FileDialogMode::Save => {
                super::bindings::cef_file_dialog_mode_t_FILE_DIALOG_SAVE
            }
        },
        &cef_title,
        &cef_initial_file_name,
        filters,
        0,
        callback as *mut super::bindings::_cef_run_file_dialog_callback_t,
    );
}
