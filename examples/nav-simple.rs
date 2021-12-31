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
    assert_eq!(header.get_rinex_type(), RinexType::NavigationMessage);
    
    // useful information
    let nb_nav_frames = rinex.len();

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
    
    //   ---> extract all Constellations
    let constellations: Vec<_> = rinex.get_record().iter()
        .filter_map(|s| Some(s["sv"].Sv().unwrap().get_constellation())).collect();
    println!("Constellation : {:#?}", constellations);

    //   ----> extract all `Sv` tied to GPS 
    let gps_vehicules: Vec<_>  = vehicules.iter()
        .filter(|s| s.Sv().unwrap().get_constellation() == Constellation::GPS)
        .collect();
    assert_eq!(gps_vehicules.len(), 0); // wait what?
    println!("GPS Vehicules : {:#?}", gps_vehicules);

    //   ----> extract all `Sv` tied to Glonass 
    let glo_vehicules: Vec<_>  = vehicules.iter()
        .filter(|s| s.Sv().unwrap().get_constellation() == Constellation::Glonass)
        .collect();
    assert_eq!(glo_vehicules.len(), nb_nav_frames); // wait what?
    println!("GLO Vehicules : {:#?}", glo_vehicules);

    //   ------> that's a GLO:NAV file 😀
    //           all `Sv` are tied to Glonass 😀😀
    assert_eq!(header.get_constellation(), Constellation::Glonass);
}
