use rinex::*;
use std::path::PathBuf;

fn main() {
    println!("RINEX: example: basic");
    let rinex = Rinex::from_file(&PathBuf::from("navsmall1.rinex")).unwrap();
    println!("{:#?}", rinex.header);
    assert_eq!(rinex.header.rinex_type, RinexType::NavigationMessage);
    assert_eq!(rinex.header.constellation, constellation::Constellation::Mixed);
    println!("{:#?}", rinex.header.version);
    println!("Record size: {:#?}", rinex.record);
}
