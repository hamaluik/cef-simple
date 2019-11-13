use std::env;
use std::path::PathBuf;

fn main() {
    // let us link the proper CEF version depending on what host we're compiling for
    let target_os = env::var("TARGET").expect("target");
    let cef_path = match target_os.as_ref() {
        "x86_64-pc-windows-msvc" => {
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("cef-x86_64-windows")
        }
        "x86_64-unknown-linux-gnu" => {
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("cef-x86_64-linux")
        }
        _ => unimplemented!(),
    };
    /*let cef_path = env::var("CEF_PATH").expect("environment variable CEF_PATH should point to the CEF distribution");
    let cef_path = PathBuf::from(cef_path);*/
    assert!(cef_path.exists());
    let cef_lib_path = cef_path.join("Release");
    let cef_lib_name = match target_os.as_ref() {
        "x86_64-pc-windows-msvc" => "libcef",
        _ => "cef",
    };

    println!("cargo:rustc-link-search={}", cef_lib_path.display());
    println!("cargo:rustc-link-lib={}", cef_lib_name);

    // since CEF is a C / C++ library we need bindings for it
    // let's generate those now if they don't already exist in our source tree
    let bindings_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src")
        .join("imp")
        .join("bindings.rs");
    if !bindings_path.exists() {
        let bindings = bindgen::Builder::default()
            .header("cefwrapper.h")
            .clang_arg(format!(
                "-I{}",
                cef_path
                    .to_str()
                    .expect("could not format path as string")
                    .to_owned()
            ))
            .derive_default(true)
            .layout_tests(false)
            .generate_comments(false)
            .whitelist_recursively(true)
            // these are all the types / functions / vars that we need directly
            .whitelist_type("cef_app_t")
            .whitelist_type("cef_base_ref_counted_t")
            .whitelist_type("cef_browser_view_delegate_t")
            .whitelist_type("cef_client_t")
            .whitelist_type("cef_context_menu_handler_t")
            .whitelist_type("cef_life_span_handler_t")
            .whitelist_type("cef_request_handler_t")
            .whitelist_type("cef_browser_t")
            .whitelist_type("cef_context_menu_handler_t")
            .whitelist_type("cef_context_menu_params_t")
            .whitelist_type("cef_frame_t")
            .whitelist_type("cef_menu_model_t")
            .whitelist_type("cef_life_span_handler_t")
            .whitelist_type("cef_binary_value_t")
            .whitelist_type("cef_errorcode_t")
            .whitelist_type("cef_request_callback_t")
            .whitelist_type("cef_request_handler_t")
            .whitelist_type("cef_sslinfo_t")
            .whitelist_type("cef_x509certificate_t")
            .whitelist_type("cef_browser_settings_t")
            .whitelist_type("cef_browser_view_create")
            .whitelist_type("cef_browser_view_delegate_t")
            .whitelist_type("cef_client_t")
            .whitelist_type("cef_image_t")
            .whitelist_type("cef_panel_t")
            .whitelist_type("cef_size_t")
            .whitelist_type("cef_string_t")
            .whitelist_type("cef_view_t")
            .whitelist_type("cef_window_delegate_t")
            .whitelist_type("cef_window_t")
            .whitelist_type("cef_main_args_t")
            .whitelist_type("cef_settings_t")
            .whitelist_function("cef_initialize")
            .whitelist_function("cef_browser_view_create")
            .whitelist_function("cef_execute_process")
            .whitelist_function("cef_quit_message_loop")
            .whitelist_function("cef_dictionary_value_create")
            .whitelist_function("cef_image_create")
            .whitelist_function("cef_request_context_get_global_context")
            .whitelist_function("cef_string_utf8_to_utf16")
            .whitelist_function("cef_run_message_loop")
            .whitelist_function("cef_shutdown")
            .whitelist_function("cef_window_create_top_level")
            .whitelist_var("cef_log_severity_t_LOGSEVERITY_INFO")
            .generate()
            .expect("Unable to generate bindings");
        let bindings = bindings.to_string();

        use std::io::prelude::*;
        use std::io::BufWriter;
        let file = std::fs::File::create(&bindings_path).expect("couldn't create bindings file!");
        let mut writer = BufWriter::new(file);
        // disable warnings and lints on the bindings file so we don't
        // get barraged by it when we run lints / fmts
        writer
            .write_all(
                r##"#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]

"##
                .as_bytes(),
            )
            .expect("failed to write bindings header!");
        writer
            .write_all(&bindings.as_bytes())
            .expect("failed to write bindings!");
    }
}
