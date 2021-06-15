use std::env;
use std::path::PathBuf;

fn main() {
    // let us link the proper CEF version depending on what host we're compiling for
    let target_os = env::var("TARGET").expect("target");
    let cef_lib_name = match target_os.as_ref() {
        "x86_64-pc-windows-msvc" => "libcef",
        _ => "cef",
    };
    println!("cargo:rustc-link-lib={}", cef_lib_name);

    let cef_path: Result<PathBuf, _> = env::var("CEF_PATH").map(From::from);
    let cef_path = cef_path.expect("CEF_PATH must be set");

    let cef_lib_path = cef_path.join("Release");
    assert!(cef_path.exists());
    println!("cargo:rustc-link-search={}", cef_lib_path.display());

    // since CEF is a C / C++ library we need bindings for it
    //// let's generate those now if they don't already exist in our source tree
    let bindings_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
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
            .opaque_type("_IMAGE_TLS_DIRECTORY64")
            .no_default("tagMONITORINFOEXA")
            .generate()
            .expect("Unable to generate bindings");
        let bindings = bindings.to_string();

        use std::io::prelude::*;
        use std::io::BufWriter;
        let file = std::fs::File::create(&bindings_path).expect("couldn't create bindings file!");
        let mut writer = BufWriter::new(file);
        writer
            .write_all(&bindings.as_bytes())
            .expect("failed to write bindings!");
    }
}
