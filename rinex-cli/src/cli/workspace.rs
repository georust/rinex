//! Workspace definition and helper
use std::{
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
    process::Command,
};

use crate::cli::Cli;

/// Workspace, describes past and current session
pub struct Workspace {
    /// Root Fullpath for this session
    pub root: PathBuf,
}

impl Workspace {
    /// Builds a new workspace either
    ///  1. from $RINEX_WORKSPACE environment variable
    ///  2. from -w workspace CLI argument
    ///  3. or defaults to ./WORSPACE, that exists within this Git repo.
    /// Refer to Wiki Pages.
    pub fn new(session: &str, cli: &Cli) -> Self {
        let root = match std::env::var("RINEX_WORKSPACE") {
            Ok(path) => Path::new(&path).join(session).to_path_buf(),
            _ => match cli.matches.get_one::<PathBuf>("workspace") {
                Some(path) => Path::new(path).join(session).to_path_buf(),
                None => Path::new("WORKSPACE").join(session).to_path_buf(),
            },
        };
        // make sure workspace does exists, otherwise create it
        create_dir_all(&root).unwrap_or_else(|e| {
            panic!(
                "failed to create session workspace \"{}\": {}",
                root.display(),
                e
            )
        });
        info!("session workspace is \"{}\"", root.to_string_lossy());
        Self {
            root: root.to_path_buf(),
        }
    }
    /// Creates subdirectory within self.
    /// Will panic on write permission issues.
    pub fn create_subdir(&self, dir: &str) {
        create_dir_all(self.root.join(dir)).unwrap_or_else(|e| {
            panic!("failed to create directory {} within workspace: {}", dir, e)
        });
    }
    /// Creates new file within this session.
    /// Will panic on write permission issues.
    pub fn create_file(&self, filename: &str) -> File {
        let fullpath = self.root.join(filename).to_string_lossy().to_string();
        let fd = File::create(&fullpath)
            .unwrap_or_else(|e| panic!("failed to create new file {}: {}", filename, e));
        info!("{} has been generated", fullpath);
        fd
    }
    /// Opens root path with prefered web browser
    #[cfg(target_os = "linux")]
    pub fn open_with_web_browser(&self) {
        let fullpath = self.root.join("index.html").to_string_lossy().to_string();
        let web_browsers = vec!["firefox", "chromium"];
        for browser in web_browsers {
            let child = Command::new(browser).arg(fullpath.clone()).spawn();
            if child.is_ok() {
                return;
            }
        }
    }
    /// Opens root path with prefered web browser
    #[cfg(target_os = "macos")]
    pub fn open_with_web_browser(&self) {
        let fullpath = self.root.join("index.html").to_string_lossy().to_string();
        Command::new("open")
            .args(&[fullpath])
            .output()
            .expect("open() failed, can't open HTML content automatically");
    }
    /// Opens root path with prefered web browser
    #[cfg(target_os = "windows")]
    pub fn open_with_web_browser(&self) {
        let fullpath = self.root.join("index.html").to_string_lossy().to_string();
        Command::new("cmd")
            .arg("/C")
            .arg(format!(r#"start {}"#, fullpath))
            .output()
            .expect("failed to open generated HTML content");
    }
}
