//use std::str::FromStr;

/// Reference is the `File Reference` Description field
#[derive(Debug, Clone)]
pub struct Reference {
    /// Organization(s) providing / gathering file content
    pub description: String,
    /// Brief description of the input used to generate the solution
    pub input: String,
    /// Description of the file contents
    pub output: String,
    /// Address of the relevant contact (email..)
    pub contact: String,
    /// Software used to generate this file
    pub software: String,
    /// Hardware used to genreate this file
    pub hardware: String,
}

impl Reference {
    pub fn with_description (&self, description: &str) -> Self {
        Self {
            description: description.to_string(),
            input: self.input.clone(),
            output: self.output.clone(),
            contact: self.contact.clone(),
            software: self.software.clone(),
            hardware: self.hardware.clone(),
        }
    }
    pub fn with_input (&self, input: &str) -> Self {
        Self {
            description: self.description.clone(),
            input: input.to_string(),
            output: self.output.clone(),
            contact: self.contact.clone(),
            software: self.software.clone(),
            hardware: self.hardware.clone(),
        }
    }
    pub fn with_output (&self, output: &str) -> Self {
        Self {
            description: self.description.clone(),
            input: self.input.clone(),
            output: output.to_string(),
            contact: self.contact.clone(),
            software: self.software.clone(),
            hardware: self.hardware.clone(),
        }
    }
    pub fn with_contact (&self, contact: &str) -> Self {
        Self {
            description: self.description.clone(),
            input: self.input.clone(),
            output: self.output.clone(),
            contact: contact.to_string(),
            software: self.software.clone(),
            hardware: self.hardware.clone(),
        }
    }
    pub fn with_software (&self, software: &str) -> Self {
        Self {
            description: self.description.clone(),
            input: self.input.clone(),
            output: self.output.clone(),
            contact: self.contact.clone(),
            software: software.to_string(),
            hardware: self.hardware.clone(),
        }
    }
    pub fn with_hardware (&self, hardware: &str) -> Self {
        Self {
            description: self.description.clone(),
            input: self.input.clone(),
            output: self.output.clone(),
            contact: self.contact.clone(),
            software: self.software.clone(),
            hardware: hardware.to_string(),
        }
    }
}

impl Default for Reference {
    fn default() -> Self {
        Self {
            description: String::from("?"),
            input: String::from("?"),
            output: String::from("?"),
            contact: String::from("unknown"),
            software: String::from("unknown"),
            hardware: String::from("unknown"),
        }
    }
}
