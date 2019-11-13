use std::mem::size_of;
use std::ptr::null_mut;

mod imp;
use imp::bindings::{
    cef_app_t, cef_execute_process, cef_initialize, cef_log_severity_t_LOGSEVERITY_ERROR,
    cef_log_severity_t_LOGSEVERITY_INFO, cef_main_args_t, cef_run_message_loop, cef_settings_t,
    cef_shutdown, cef_window_create_top_level, cef_window_delegate_t,
};
pub use imp::window_delegate::WindowOptions;
use imp::{app, window_delegate};

pub struct Cef {}

impl Cef {
    #[cfg(unix)]
    pub fn initialize(debug_port: Option<u16>, disable_command_line_args: bool) -> Result<Cef, Box<dyn std::error::Error>> {
        use std::ffi::CString;
        use std::os::raw::{c_char, c_int};
        let args: Vec<CString> = std::env::args().map(|x| CString::new(x).unwrap()).collect();
        let args: Vec<*mut c_char> = args.iter().map(|x| x.as_ptr() as *mut c_char).collect();
        let main_args = cef_main_args_t {
            argc: args.len() as c_int,
            argv: args.as_ptr() as *mut *mut c_char,
        };

        log::debug!("preparing app");
        let app = app::allocate();

        log::debug!("executing process");
        let exit_code = unsafe {
            (*app).inc_ref();
            cef_execute_process(&main_args, app as *mut cef_app_t, null_mut())
        };
        if exit_code >= 0 {
            std::process::exit(exit_code);
        }

        let mut settings = cef_settings_t::default();
        settings.size = size_of::<cef_settings_t>();
        settings.no_sandbox = 1;
        if let Some(port) = debug_port {
            settings.remote_debugging_port = port as i32;
        }
        settings.command_line_args_disabled = if disable_command_line_args { 1 } else { 0 };
        if cfg!(debug_assertions) {
            settings.log_severity = cef_log_severity_t_LOGSEVERITY_INFO;
        } else {
            settings.log_severity = cef_log_severity_t_LOGSEVERITY_ERROR;
        }

        log::debug!("initializing");
        unsafe {
            (*app).inc_ref();
            if cef_initialize(&main_args, &settings, app as *mut cef_app_t, null_mut()) != 1 {
                return Err(Box::from("failed to initialize"));
            }
        }

        Ok(Cef {})
    }

    #[cfg(windows)]
    pub fn initialize(debug_port: Option<u16>, disable_command_line_args: bool) -> Result<Cef, Box<dyn std::error::Error>> {
        let main_args = unsafe {
            cef_main_args_t {
                instance: winapi::um::libloaderapi::GetModuleHandleA(null_mut())
                    as imp::bindings::HINSTANCE,
            }
        };

        log::debug!("preparing app");
        let app = app::allocate();

        log::debug!("executing process");
        let exit_code = unsafe {
            (*app).inc_ref();
            cef_execute_process(&main_args, app as *mut cef_app_t, null_mut())
        };
        if exit_code >= 0 {
            std::process::exit(exit_code);
        }

        let mut settings = cef_settings_t::default();
        settings.size = size_of::<cef_settings_t>();
        settings.no_sandbox = 1;
        if let Some(port) = debug_port {
            settings.remote_debugging_port = port as i32;
        }
        settings.command_line_args_disabled = if disable_command_line_args { 1 } else { 0 };
        if cfg!(debug_assertions) {
            settings.log_severity = cef_log_severity_t_LOGSEVERITY_INFO;
        } else {
            settings.log_severity = cef_log_severity_t_LOGSEVERITY_ERROR;
        }

        log::debug!("initializing");
        unsafe {
            (*app).inc_ref();
            if cef_initialize(&main_args, &settings, app as *mut cef_app_t, null_mut()) != 1 {
                return Err(Box::from("failed to initialize"));
            }
        }

        Ok(Cef {})
    }

    pub fn open_window(&self, options: WindowOptions) -> Result<(), Box<dyn std::error::Error>> {
        let window_delegate = window_delegate::allocate(options);
        log::debug!("creating window");
        let _window = unsafe {
            (*window_delegate).inc_ref();
            cef_window_create_top_level(window_delegate as *mut cef_window_delegate_t)
        };

        Ok(())
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("running message loop");
        unsafe { cef_run_message_loop() };

        log::debug!("shutting down");
        unsafe { cef_shutdown() };

        Ok(())
    }
}
