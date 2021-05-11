use cef_simple::{Cef, WindowOptions};
use simplelog::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cef = Cef::initialize(Some(8000), false)?;

    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    let page = urlencoding::encode(include_str!("page.html"));

    cef.open_window(WindowOptions {
        url: format!("data:text/html,{}", page),
        title: Some("CEF Simple—On-Screen-Keyboard Demo".to_string()),
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
