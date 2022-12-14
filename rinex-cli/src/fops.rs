use rinex::prelude::*;
use std::process::Command;

/* Returns filename prefix, from given content */
pub fn filename(fp: &str) -> String {
    let content: Vec<&str> = fp.split("/").collect();
    let content: Vec<&str> = content[content.len() - 1].split(".").collect();
    content[0].to_string()
}

/* Returns .XXX for given filename */
pub fn suffix(fp: &str) -> String {
    let content: Vec<&str> = fp.split(".").collect();
    content[content.len() - 1].to_string()
}

#[cfg(target_os = "linux")]
pub fn open_html_with_default_app(path: &str) {
    let web_browsers = vec!["firefox", "chromium"];
    for browser in web_browsers {
        let child = Command::new(browser).args([path]).spawn();
        if child.is_ok() {
            return;
        }
    }
}

#[cfg(target_os = "macos")]
pub fn open_html_with_default_app(path: &str) {
    Command::new("open")
        .args(&[path])
        .output()
        .expect("open() failed, can't open HTML content automatically");
}

#[cfg(target_os = "windows")]
pub fn open_html_with_default_app(path: &str) {
    Command::new("cmd")
        .arg("/C")
        .arg(format!(r#"start {}"#, path))
        .output()
        .expect("failed to open generated HTML content");
}

/// Macro to generate a file from an input RINEX
/// with possible control over the file name, from the command line
pub fn generate(rnx: &Rinex, path: &str) -> Result<(), rinex::Error> {
    rnx.to_file(path)?;
    info!("\"{}\" has been generated", path);
    Ok(())
}
