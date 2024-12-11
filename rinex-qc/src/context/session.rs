use crate::context::{Error, QcContext};

use std::fs::{create_dir_all, File};

use std::process::Command;

impl QcContext {
    // /// Returns the Year range of this session, in chronological order
    // fn year_range(&self) -> Option<Range<u16>> {
    //     Some(self.oldest_year()?..self.newest_year()?)
    // }

    // fn oldest_year(&self) -> Option<u16> {
    //     let min = self.all_meta_iter().map(|meta| meta.year).min();
    //     if min == 0 {
    //         None
    //     } else {
    //         Some(min)
    //     }
    // }

    // fn newest_year(&self) -> Option<u16> {
    //     let max = self.all_meta_iter().map(|meta| meta.year).max();
    //     if max == 0 {
    //         None
    //     } else {
    //         Some(max)
    //     }
    // }

    // fn oldest_doy(&self, year: u16) -> Option<u16> {
    //     let min = self.all_meta_iter().filter_map(|meta| meta.year == year).min();
    //     if min == 0 {
    //         None
    //     } else {
    //         Some(min)
    //     }
    // }

    // fn newest_doy(&self, year: u16) -> Option<u16> {
    //     let max = self.all_meta_iter().filter_map(|meta| meta.year == year).max();
    //     if max == 0 {
    //         None
    //     } else {
    //         Some(max)
    //     }
    // }

    // fn doy_range(&self, year: u16) -> Option<Range<u16>> {
    //     Some(self.oldest_doy(year)?..self.newest_doy(year)?)
    // }

    // /// Prepare and deploy [QcContext], at this point
    // /// we're ready to generate data
    // fn deploy(&self) -> Result<(), Error> {
    //     // make
    // }

    // /// Creates the file hierarchy, within the workspace
    // fn create_workspace(&self) -> Result<(), Error> {
    //     for year in self.year_range() {
    //         for doy in self.doy_range(year) {
    //             for product in self.all_products_iter() {
    //                 let fp = format!("{}/{}/{:x}", year, doy, product);
    //                 self.create_subdir()?;
    //             }
    //         }
    //     }
    //     trace!("workspace tree generated");
    // }

    /// Session deployment
    pub fn deploy(&self) -> Result<(), Error> {
        create_dir_all(&self.workspace).map_err(|_| Error::IO)?;
        Ok(())
    }

    /// Create a subdir inside the workspace, usually
    /// to generate output products.
    pub fn create_subdir(&self, subdir: &str) -> Result<(), Error> {
        create_dir_all(self.workspace.join(subdir)).map_err(|_| Error::IO)?;
        Ok(())
    }

    /// Create a file inside the workspace and return [File] handle
    pub fn create_file(&self, name: &str) -> Result<File, Error> {
        let  fd = File::create(self.workspace.join(name)).map_err(|_| Error::IO)?;
        Ok(fd)
    }

    /// Utilities to open selected path, with prefered web browser
    /// Opens root path with prefered web browser
    #[cfg(target_os = "linux")]
    pub fn open_with_web_browser(&self) {
        let fullpath = self.workspace.to_string_lossy().to_string();
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
        let fullpath = self.workspace.to_string_lossy().to_string();
        Command::new("open")
            .args(&[fullpath])
            .output()
            .expect("open() failed, can't open HTML content automatically");
    }

    /// Opens root path with prefered web browser
    #[cfg(target_os = "windows")]
    pub fn open_with_web_browser(&self) {
        let fullpath = self.workspace.to_string_lossy().to_string();
        Command::new("cmd")
            .arg("/C")
            .arg(format!(r#"start {}"#, fullpath))
            .output()
            .expect("failed to open generated HTML content");
    }

    /// Opens root path with prefered web browser
    #[cfg(target_os = "android")]
    pub fn open_with_web_browser(&self) {
        let fullpath = self.workspace.to_string_lossy().to_string();
        Command::new("xdg-open")
            .args(&[fullpath])
            .output()
            .expect("failed to open generated HTML content");
    }
}
