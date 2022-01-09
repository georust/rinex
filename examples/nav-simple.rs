use rinex::*;
use rinex::record::*;
use rinex::constellation::*;

// .unique() is quite convenient filter
use itertools::Itertools;

// .filter() advanced filter combination
//TODO

fn main() {
    println!("RINEX: example: nav-simple");

    let navigation_file = std::path::PathBuf::from(
        env!("CARGO_MANIFEST_DIR").to_owned() + "/examples/navsmall1.rinex");
    let rinex = Rinex::from_file(&navigation_file).unwrap();

    // header informations
    let header = rinex.get_header();
    println!("Version: {:#?}", header.get_rinex_version());
    assert_eq!(header.is_crinex(), false);
    assert_eq!(header.get_rinex_type(), RinexType::NavigationMessage);
    
    // (NAV) manipulation
    //      ➡ Extract all different `Sv` encountered in this record
    let vehicules: Vec<_> = rinex.get_record()
        .iter()                   //  (1) get a (NAV) record iter  
        .map(|s| s.as_nav().unwrap()) 
            .map(|s| Some(s["sv"].as_sv())) // (2) retaining only this item
            .unique()                       // (3) unique() filter
            .collect();
    println!("`Sv` : {:#?}", vehicules);
    
    // (NAV) manipulation
    //      ➡ isolate `R03` Satellite Vehicule (Sv)
    //        1) build item to match
    let sv = Sv::new(Constellation::Glonass, 0x03);
    let to_match = RecordItem::Sv(sv); 
    let r03_data: Vec<_> = rinex.get_record()
        .iter()                   //  (2) get a (NAV) record iter  
        .map(|s| s.as_nav().unwrap()) 
            .filter(|s| s["sv"] == to_match) // (3) filter by field
            .collect();
    println!("\"R03\" filter : {:#?}", r03_data);
    
    // (NAV) manipulation
    //      ➡ extract all constellations encountered 
    let constellations: Vec<_> = rinex.get_record()
        .iter()     // (1) get (NAV) record iterator
        .map(|s| s.as_nav().unwrap())
            .map(|s| s["sv"].as_sv().unwrap().get_constellation())
            .unique() // from all `Sv` we extract the constellation field value
            .collect(); // and we iterate with .unique()
    println!("Constellation : {:#?}", constellations);

    // (NAV) manipulation
    //      ➡ extract all `Sv` tied to GPS
    let gps_data: Vec<_> = rinex.get_record()
        .iter()
        .map(|s| s.as_nav().unwrap())
            .filter(|s| s["sv"].as_sv().unwrap().get_constellation() == Constellation::GPS)
            .collect(); // from all ̀`Sv` fields we filter on the constellation value
    println!("GPS Data : {:#?}", gps_data);
    assert_eq!(gps_data.len(), 0); // None.. really? 
    
    //   ------> that's a GLO:NAV file 😀
    //           all `Sv` are tied to Glonass 😀😀
    let glo_data: Vec<_> = rinex.get_record()
        .iter()
        .map(|s| s.as_nav().unwrap())
            .filter(|s| s["sv"].as_sv().unwrap().get_constellation() == Constellation::Glonass)
            .collect(); 
    assert_eq!(glo_data.len(), rinex.len());
    assert_eq!(header.get_constellation(), Constellation::Glonass);
    println!("GLO Data : {:#?}", glo_data);
}
