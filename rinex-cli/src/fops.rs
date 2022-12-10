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
