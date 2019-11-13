use libc::{calloc, free};
use log::{error, info};
use std::mem::size_of;
use std::os::raw::{c_int, c_void};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::bindings::{
    cef_base_ref_counted_t, cef_binary_value_t, cef_browser_t, cef_errorcode_t,
    cef_request_callback_t, cef_request_handler_t, cef_sslinfo_t, cef_string_t,
    cef_x509certificate_t,
};

#[derive(Debug)]
#[repr(C)]
pub struct RequestHandler {
    request_handler: cef_request_handler_t,
    ref_count: AtomicUsize,
}

impl RequestHandler {
    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn on_certificate_error(
    _slf: *mut cef_request_handler_t,
    _browser: *mut cef_browser_t,
    _cert_error: cef_errorcode_t,
    _request_url: *const cef_string_t,
    ssl_info: *mut cef_sslinfo_t,
    callback: *mut cef_request_callback_t,
) -> ::std::os::raw::c_int {
    // check to see if it's the cert that our own server generated

    let cert_serial: Vec<u8> = unsafe {
        let cert: *mut cef_x509certificate_t = (*ssl_info).get_x509certificate.unwrap()(ssl_info);
        let serial: *mut cef_binary_value_t = (*cert).get_serial_number.unwrap()(cert);
        let serial_size: usize = (*serial).get_size.unwrap()(serial);
        let mut serial_bytes: Vec<u8> = vec![0; serial_size];
        (*serial).get_data.unwrap()(
            serial,
            serial_bytes.as_mut_ptr() as *mut core::ffi::c_void,
            serial_size,
            0,
        );
        serial_bytes
    };

    if cert_serial
        != vec![
            0x01, 0x55, 0x37, 0xfe, 0x94, 0x15, 0xf6, 0x5f, 0xe0, 0x13, 0xc3, 0x2e, 0x62, 0x8c,
            0x4b, 0xe8, 0x5a, 0xfd, 0x2a, 0x56,
        ]
    {
        error!("invalid certificate serial: {:x?}", cert_serial);
        0
    } else {
        info!("allowing ReJoyce certificate");
        unsafe {
            (*callback).cont.unwrap()(callback, 1);
        }
        1
    }
}

pub fn allocate() -> *mut RequestHandler {
    let request_handler = unsafe { calloc(1, size_of::<RequestHandler>()) as *mut RequestHandler };
    unsafe {
        (*request_handler).request_handler.base.size = size_of::<RequestHandler>();
        (*request_handler).ref_count.store(1, Ordering::SeqCst);
        (*request_handler).request_handler.base.add_ref = Some(add_ref);
        (*request_handler).request_handler.base.release = Some(release);
        (*request_handler).request_handler.base.has_one_ref = Some(has_one_ref);
        (*request_handler).request_handler.base.has_at_least_one_ref = Some(has_at_least_one_ref);

        (*request_handler).request_handler.on_certificate_error = Some(on_certificate_error);
    };

    request_handler
}

extern "C" fn add_ref(base: *mut cef_base_ref_counted_t) {
    let request_handler = base as *mut RequestHandler;
    unsafe {
        (*request_handler).ref_count.fetch_add(1, Ordering::SeqCst);
    }
}

extern "C" fn release(base: *mut cef_base_ref_counted_t) -> c_int {
    let request_handler = base as *mut RequestHandler;
    let count = unsafe { (*request_handler).ref_count.fetch_sub(1, Ordering::SeqCst) - 1 };

    if count == 0 {
        unsafe {
            free(request_handler as *mut c_void);
        }
        1
    } else {
        0
    }
}

extern "C" fn has_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let request_handler = base as *mut RequestHandler;
    let count = unsafe { (*request_handler).ref_count.load(Ordering::SeqCst) };
    if count == 1 {
        1
    } else {
        0
    }
}

extern "C" fn has_at_least_one_ref(base: *mut cef_base_ref_counted_t) -> c_int {
    let request_handler = base as *mut RequestHandler;
    let count = unsafe { (*request_handler).ref_count.load(Ordering::SeqCst) };
    if count >= 1 {
        1
    } else {
        0
    }
}
