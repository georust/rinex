use rinex::*;
use rinex::record::*;
use rinex::constellation::Constellation;

fn main() {
    println!("RINEX: example: nav-simple");

    let navigation_file = std::path::PathBuf::from(
        env!("CARGO_MANIFEST_DIR").to_owned() + "/data/NAV/amel0010.21g");
    let rinex = Rinex::from_file(&navigation_file).unwrap();

    // header informations
    let header = rinex.get_header();
    println!("Version: {:#?}", header.get_rinex_version());
    assert_eq!(header.is_crinex(), false);
    assert_eq!(header.get_constellation(), Constellation::Glonass);
    assert_eq!(header.get_rinex_type(), RinexType::NavigationMessage);
    
    // rinex obj
    assert_eq!(rinex.len(), 335);

    // (NAV) manipulation
    //let records = rinex.get_records().iter()
    //    .filter();

    if let RinexRecord::RinexNavRecord(nav) = rinex.get_record(0) {
        let sv = &nav.items["sv"];
        println!("Sv {:#?}", sv);
    }
}
