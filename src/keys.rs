use crate::version::Version;
use crate::header::RinexType;

/// Maximal Key listing ever
pub const KeyBankMaxSize: usize = 64;

type KeyBankItem = (String, String); // Key, Type

#[derive(Debug)]
pub struct KeyBank {
    keys: Vec<KeyBankItem> // key, type
}

impl KeyBank {
    /// Builds known list of item keys
    /// for this particular Rinex release & type
    pub fn new (version: &Version, rtype: &RinexType) -> Result<KeyBank, std::io::Error> {
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

        let mut type_matched = false;
        let type_to_match = rtype.to_string();

        line = lines.next() // skip header
            .unwrap()
            .trim();

        loop {
            
            if type_matched {
                if version_matched {
                    let cleanedup = line.replace(":","");
                    let cleanedup = cleanedup.replace(",","");
                    let cleanedup = cleanedup.replace("\"","");
                    let items: Vec<&str> = cleanedup.split_ascii_whitespace()
                        .collect();
                    println!("ITEMS \"{}\" \"{}\"", items[0], items[1]);
                    keys.push((String::from(items[0]),String::from(items[1])))
                } else {
                    version_matched = line.contains(&version_to_match)
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
