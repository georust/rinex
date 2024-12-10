use std::fs::create_dir_all;
use crate::context::{QcContext, Error};

impl QcContext {
    
    /// Utility to create a directory within this session
    fn create_subdir(&self, directory: &str) -> Result<(), Error> {
        let fullpath = self.workspace.join(directory);
        create_dir_all(
            self.workspace
        )
    }
}
