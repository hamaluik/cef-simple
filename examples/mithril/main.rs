use cef_simple::{Cef, WindowOptions};
use simplelog::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
    )])
    .unwrap();

    let cef = Cef::initialize(Some(8000), false)?;

    let page = urlencoding::encode(include_str!("index.html"));

    cef.open_window(WindowOptions {
        url: format!("data:text/html,{}", page),
        title: Some("CEF Mithril Demo".to_string()),
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
