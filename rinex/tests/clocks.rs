#[cfg(test)]
mod test {
    use rinex::*;
    use std::str::FromStr;
    use std::process::Command;
    #[test]
    fn v3_demo() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/CLK/V3/demo.txt";
        let rinex = Rinex::from_file(&test_resource);
        println!("{:#?}", rinex.unwrap().header.clocks);
        //assert_eq!(rinex.is_ok(), true);
        //let rinex = rinex.unwrap();
        //assert_eq!(rinex.is_clocks_rinex(), true);
    }

    #[test]
    fn is_new_epoch() {
        let l = "AR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(clocks::is_new_epoch(l), true);
        let l = "RA AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(clocks::is_new_epoch(l), false);
        let l = "DR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(clocks::is_new_epoch(l), true);
        let l = "CR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(clocks::is_new_epoch(l), true);
        let l = "AS AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(clocks::is_new_epoch(l), true);
    }
}
