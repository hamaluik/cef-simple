use cef_simple::{Cef, WindowOptions};
use simplelog::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cef = Cef::initialize(None, true)?;

    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
    )
    .unwrap()])
    .unwrap();

    let page = include_str!("page.html");

    cef.open_window(WindowOptions {
        url: format!("data:text/html,{}", page),
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
