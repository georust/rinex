use rinex::*;
use rinex::record::*;
use rinex::constellation::*;

fn main() {
    println!("RINEX: example: nav-simple");

    let navigation_file = std::path::PathBuf::from(
        env!("CARGO_MANIFEST_DIR").to_owned() + "/examples/navsmall1.rinex");
    let rinex = Rinex::from_file(&navigation_file).unwrap();

    // header informations
    let header = rinex.get_header();
    println!("Version: {:#?}", header.get_rinex_version());
    assert_eq!(header.is_crinex(), false);
    assert_eq!(header.get_constellation(), Constellation::Glonass);
    assert_eq!(header.get_rinex_type(), RinexType::NavigationMessage);
    
    // rinex obj
    assert_eq!(rinex.len(), 6);

    // (NAV) manipulation
    //   --> isolate `R03` Satellite Vehicule (Sv)
    let sv = Sv::new(Constellation::Glonass, 0x03);
    let to_match = RecordItem::Sv(sv); 
    let matching: Vec<_> = rinex.get_record().iter()
        .filter(|s| s["sv"] == to_match)
        .collect();

    println!("\"R03\" filter : {:#?}", matching);
    
    //   ---> extract all Sv 
    let vehicules: Vec<_> = rinex.get_record().iter()
        .map(|s| s["sv"]).collect();
    println!("Extracting all `Sv` : {:#?}", vehicules);

    //   ----> identify which ones are tied to GPS
    //let gps_vehicules: Vec<_> = vehicules.iter()
    //    .collect();

    // locate Sv tied to Constellation::GPS 
    //let to_match = Constellation::GPS;
    //let matching = vehicules
    //    .filter(|s| s == to_match);
    
    //   ----> determine which ones are tied to GPS
    
    //   GLONASS:NAV `G` isolation isn't fruitful..

}
