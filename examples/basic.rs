use rinex::*;
use std::path::PathBuf;

fn main() {
    println!("RINEX: example: basic");
    let rinex = Rinex::from_file(&PathBuf::from("navsmall1.rinex")).unwrap();
    let header = rinex.get_header();
    println!("{:#?}", header);
    assert_eq!(header.get_rinex_type(), RinexType::NavigationMessage);
    assert_eq!(header.get_constellation(), constellation::Constellation::Mixed);
    println!("{:#?}", header.get_rinex_version());
    println!("Record size: {}", rinex.len());
}
