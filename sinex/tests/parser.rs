#[cfg(test)]
mod test {
    use sinex::*;
    #[test]
    fn test_parser() {
        let test_resources = env!("CARGO_MANIFEST_DIR").to_owned() + "/data/";
        let test_data = vec!["BIA"];
        for data in test_data {
            let data_path = std::path::PathBuf::from(test_resources.to_owned() + data);
            for revision in std::fs::read_dir(data_path).unwrap() {
                let rev = revision.unwrap();
                let rev_path = rev.path();
                let rev_fullpath = &rev_path.to_str().unwrap();
                for entry in std::fs::read_dir(rev_fullpath).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    let full_path = &path.to_str().unwrap();
                    let is_hidden = entry.file_name().to_str().unwrap().starts_with('.');
                    println!("Parsing file: \"{}\"", full_path);
                    if !is_hidden {
                        // PARSER
                        let sinex = Sinex::from_file(full_path);
                        //assert_eq!(sinex.is_ok(), true);
                        let sinex = sinex.unwrap();
                        println!("{:#?}", sinex.header);
                        // RECORD
                        match data {
                            "BIA" => { // Bias solutions record
                            },
                            _ => {},
                        }
                    }
                }
            }
        }
    }
}
