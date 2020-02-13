use std::ffi::CString;
use std::mem::size_of;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{
    cef_base_ref_counted_t, cef_browser_settings_t, cef_browser_view_create,
    cef_browser_view_delegate_t, cef_client_t, cef_dictionary_value_create, cef_image_create,
    cef_image_t, cef_panel_delegate_t, cef_panel_t, cef_request_context_get_global_context,
    cef_size_t, cef_state_t_STATE_DISABLED, cef_string_t, cef_string_utf8_to_utf16,
    cef_view_delegate_t, cef_view_t, cef_window_delegate_t, cef_window_t,
};
use super::{browser_view_delegate, client};

#[derive(Debug)]
pub struct WindowOptions {
    pub url: String,
    pub title: Option<String>,
    pub maximized: bool,
    pub fullscreen: bool,
    pub size: Option<(i32, i32)>,
    pub window_icon: Option<&'static [u8]>,
    pub window_app_icon: Option<&'static [u8]>,
}

impl Default for WindowOptions {
    fn default() -> WindowOptions {
        WindowOptions {
            url: "https://hamaluik.ca/".to_string(),
            title: None,
            maximized: false,
            fullscreen: false,
            size: Some((1280, 720)),
            window_icon: None,
            window_app_icon: None,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct WindowDelegate {
    pub window_delegate: cef_window_delegate_t,
    pub ref_count: AtomicUsize,
    pub options: WindowOptions,
    pub window_icon: Option<*mut cef_image_t>,
    pub window_app_icon: Option<*mut cef_image_t>,
}

impl WindowDelegate {
    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn is_frameless(_: *mut cef_window_delegate_t, _: *mut cef_window_t) -> c_int {
    0
}

extern "C" fn can_resize(_: *mut cef_window_delegate_t, _: *mut cef_window_t) -> c_int {
    1
}

extern "C" fn can_maximize(_: *mut cef_window_delegate_t, _: *mut cef_window_t) -> c_int {
    1
}

extern "C" fn can_minimize(_: *mut cef_window_delegate_t, _: *mut cef_window_t) -> c_int {
    1
}

extern "C" fn can_close(_: *mut cef_window_delegate_t, _: *mut cef_window_t) -> c_int {
    1
}

extern "C" fn window_delegate_created(slf: *mut cef_window_delegate_t, window: *mut cef_window_t) {
    log::debug!("window delegate created!");

    let window_delegate = slf as *mut WindowDelegate;
    let mut cef_url = cef_string_t::default();
    unsafe {
        let url = (*window_delegate).options.url.as_bytes();
        let url = CString::new(url).unwrap();
        cef_string_utf8_to_utf16(url.as_ptr(), url.to_bytes().len() as u64, &mut cef_url);
    }

    let mut browser_settings = cef_browser_settings_t::default();
    browser_settings.databases = cef_state_t_STATE_DISABLED;
    browser_settings.local_storage = cef_state_t_STATE_DISABLED;
    browser_settings.application_cache = cef_state_t_STATE_DISABLED;

    let client = client::allocate();
    let browser_view_delegate = browser_view_delegate::allocate();

    let browser_view = unsafe {
        (*client).inc_ref();
        (*browser_view_delegate).inc_ref();
        cef_browser_view_create(
            client as *mut cef_client_t,
            &cef_url,
            &browser_settings,
            cef_dictionary_value_create(),
            cef_request_context_get_global_context(),
            browser_view_delegate as *mut cef_browser_view_delegate_t,
        )
    };

    unsafe {
        (*browser_view).base.base.add_ref.unwrap()(browser_view as *mut cef_base_ref_counted_t);
        (*window).base.add_child_view.unwrap()(
            window as *mut cef_panel_t,
            browser_view as *mut cef_view_t,
        );

        (*window).show.unwrap()(window);

        if let Some(title) = &(*window_delegate).options.title {
            let mut cef_title = cef_string_t::default();
            let title = title.as_bytes();
            let title = CString::new(title).unwrap();
            cef_string_utf8_to_utf16(
                title.as_ptr(),
                title.to_bytes().len() as u64,
                &mut cef_title,
            );

            (*window).set_title.unwrap()(window, &cef_title);
        }

        if let Some(icon) = (*window_delegate).window_icon {
            (*window).set_window_icon.unwrap()(window, icon);
        }

        if let Some(icon) = (*window_delegate).window_app_icon {
            (*window).set_window_app_icon.unwrap()(window, icon);
        }

        if let Some(size) = (*window_delegate).options.size {
            let size: cef_size_t = cef_size_t {
                width: size.0,
                height: size.1,
            };
            (*window).center_window.unwrap()(window, &size);
        }

        if (*window_delegate).options.maximized {
            (*window).maximize.unwrap()(window);
        }

        if (*window_delegate).options.fullscreen {
            (*window).set_fullscreen.unwrap()(window, 1);
        }
    }
}

pub unsafe fn allocate(options: WindowOptions) -> *mut WindowDelegate {
    let window_icon = if let Some(data) = options.window_icon {
        let image = cef_image_create();
        (*image).add_png.unwrap()(image, 1.0, data.as_ptr() as *const _, data.len() as u64);
        Some(image)
    } else {
        None
    };

    let window_app_icon = if let Some(data) = options.window_app_icon {
        let image = cef_image_create();
        (*image).add_png.unwrap()(image, 1.0, data.as_ptr() as *const _, data.len() as u64);
        Some(image)
    } else {
        None
    };

    let window_delegate = WindowDelegate {
        window_delegate: cef_window_delegate_t {
            base: cef_panel_delegate_t {
                base: cef_view_delegate_t {
                    base: cef_base_ref_counted_t {
                        size: size_of::<WindowDelegate>() as u64,
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
                },
            },
            on_window_created: Some(window_delegate_created),
            on_window_destroyed: None,
            get_parent_window: None,
            is_frameless: Some(is_frameless),
            can_resize: Some(can_resize),
            can_maximize: Some(can_maximize),
            can_minimize: Some(can_minimize),
            can_close: Some(can_close),
            on_accelerator: None,
            on_key_event: None,
        },
        ref_count: AtomicUsize::new(1),
        options,
        window_icon,
        window_app_icon,
    };

    Box::into_raw(Box::from(window_delegate))
}

extern "C" fn add_ref(base: *mut cef_base_ref_counted_t) {
    let window_delegate = base as *mut WindowDelegate;
    unsafe {
        (*window_delegate).ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn release(base: *mut cef_base_ref_counted_t) -> c_int {
    let window_delegate = base as *mut WindowDelegate;
    let count = unsafe { (*window_delegate).ref_count.fetch_sub(1, Ordering::SeqCst) - 1 };

    if count == 0 {
        unsafe {
            Box::from_raw(window_delegate);
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let window_delegate = base as *mut WindowDelegate;
    let count = unsafe { (*window_delegate).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let window_delegate = base as *mut WindowDelegate;
    let count = unsafe { (*window_delegate).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
