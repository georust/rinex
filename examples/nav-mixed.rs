use rinex::*;
use rinex::record::*;
use rinex::constellation::*;

fn main() {
    println!("RINEX: example: nav-mixed");

    let navigation_file = std::path::PathBuf::from(
        env!("CARGO_MANIFEST_DIR").to_owned() + "/examples/navsmall2.rinex");
    let rinex = Rinex::from_file(&navigation_file).unwrap();

    // header informations
    let header = rinex.get_header();
    println!("Version: {:#?}", header.get_rinex_version());
    assert_eq!(header.is_crinex(), false);
    assert_eq!(header.get_rinex_type(), RinexType::NavigationMessage);
    
    assert_eq!(header.get_constellation(), Constellation::Mixed); 
    // ----> 😢😢 
    //       this isꞥŧ going to be easy 😢
    
    // useful information
    let nb_nav_frames = rinex.len();

    // (NAV) manipulation
    //   --> do we have some Glonass? 
    let vehicules: Vec<_> = rinex.get_record().iter()
        .map(|s| s["sv"]).collect();
    let glo_vehicules: Vec<_> = vehicules.iter()
        .filter(|s| s.Sv().unwrap().get_constellation() == Constellation::Glonass)
        .collect();
    assert_eq!(glo_vehicules.len(), 0);
    
    // ----> no Glonass? 
    //       😢😢 what else? 
    let gal_vehicules: Vec<_> = vehicules.iter()
        .filter(|s| s.Sv().unwrap().get_constellation() == Constellation::Galileo)
        .collect();
    assert_eq!(gal_vehicules.len(), 5);
    
    // ----> Cool we have something ! 
    //       😢😢 what is that "europe"? 😀
    //       how good is this thing doing

    // -----> keys.json [NavigationMessage][V3][GAL]
}
