use std::io::BufRead;

pub struct HtmlDiv<'a> {
    name: &'a str,
    start_pos: usize,
    end_pos: usize,
}

pub fn find<'a, BR: BufRead>(r: &mut BR, div: &str) -> Option<HtmlDiv<'a>> {
    let mut inside_div = false;
    let mut ret = Option::<HtmlDiv>::None;
    let mut buf = String::with_capacity(1024);
    while let Ok(size) = r.read_line(&mut buf) {
        if let Some(start) = buf.find("div=\"") {
            if let Some(end) = buf[start..].find("\"") {
                let name = &buf[start + 1..end];
            }
        }
    }
    ret
}

#[cfg(test)]
mod test {
    use super::{find, HtmlDiv};
    #[test]
    fn test_div_finder_simple() {
        let content = "
        ";
    }
}
