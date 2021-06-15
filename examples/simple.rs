use cef_simple::{Cef, WindowOptions};
use simplelog::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    let cef = Cef::initialize(None, true)?;

    cef.open_window(WindowOptions {
        url: "https://www.rust-lang.org/".to_owned(),
        title: Some("CEF Simple".to_string()),
        window_icon: Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/icon.png"
        ))),
        window_app_icon: Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/icon.png"
        ))),
        ..WindowOptions::default()
    })?;

    cef.run()?;

    Ok(())
}
