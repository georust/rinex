use rinex::*;

fn main() {
    println!("RINEX: example: nav-simple");

    let navigation_file = std::path::PathBuf::from(
        env!("CARGO_MANIFEST_DIR").to_owned() + "/data/NAV/amel0010.21g");
    let rinex = Rinex::from_file(&navigation_file).unwrap();
    println!("{:#?}", rinex);

    // header informations
    let header = rinex.get_header();
    println!("Is Crinex: {}", header.is_crinex());
    println!("Version: {:?}", header.get_rinex_version());
    println!("Constellation: {:?}", header.get_constellation());
    println!("Type: {:?}", header.get_rinex_type());

    // rinex obj
    println!("Nb NAV records: {}", rinex.len());
}
