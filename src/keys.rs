use crate::version::Version;
use crate::header::RinexType;
use crate::constellation::Constellation;

/// Biggest Key listing ever
pub const KeyBankMaxSize: usize = 64;

/// Keybank item alias
type KeyBankItem = (String, String); // Key, Type

#[derive(Debug)]
pub struct KeyBank {
    pub keys: Vec<KeyBankItem> // key, type
}

impl KeyBank {
    /// Builds known list of item keys
    /// for this particular Rinex release & type
    pub fn new (version: &Version, rtype: &RinexType, constel: &Constellation) -> Result<KeyBank, std::io::Error> {
        let mut keys: Vec<KeyBankItem> = Vec::with_capacity(KeyBankMaxSize);
        let key_listing = std::path::PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned()
                + "/keys.json");
        let content = std::fs::read_to_string(&key_listing)?;
        let mut lines = content.lines();
        let mut line = lines.next()
            .unwrap();
        
        let mut version_matched = false;
        let version_to_match = format!("V{}", version.get_major());
        let mut constel_matched = false;
        let constel_to_match = constel.to_string();
        let mut type_matched = false;
        let type_to_match = rtype.to_string();

        line = lines.next() // skip header
            .unwrap()
            .trim();

        loop {
            
            if type_matched {
                if constel_matched {
                    if version_matched {
                        if line.contains("}") {
                            break // DONE
                        }

                        let cleanedup = line.replace(":","");
                        let cleanedup = cleanedup.replace(",","");
                        let cleanedup = cleanedup.replace("\"","");
                        let items: Vec<&str> = cleanedup.split_ascii_whitespace()
                            .collect();
                        keys.push((String::from(items[0]),String::from(items[1])))
                    } else {
                        version_matched = line.contains(&version_to_match)
                    }
                } else {
                    constel_matched = line.contains(&constel_to_match)    
                }
            } else {
                type_matched = line.contains(&type_to_match)
            }

            if let Some(l) = lines.next() {
                line = l.trim()
            } else {
                break
            }
        }
        Ok(KeyBank{keys})
    }
}
