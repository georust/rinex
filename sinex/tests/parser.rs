#[cfg(test)]
mod test {
    use sinex::*;
    #[test]
    fn test_parser() {
        let resources = env!("CARGO_MANIFEST_DIR")
            .to_owned()
            + "/data";
        let resources = std::path::PathBuf::from(resources);
        for entry in std::fs::read_dir(resources).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let full_path = &path.to_str()
                .unwrap();
            let is_hidden = entry
                .file_name()
                .to_str()
                .unwrap()
                .starts_with(".");
            if !is_hidden {
                println!("PARSING {}", full_path);
                let sinex = Sinex::from_file(full_path);
                println!("{:#?}", sinex);
            }
        }
    }
}
