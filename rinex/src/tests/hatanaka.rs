use std::{
    fs::File;,
    io::Read,
};

use crate::hatanaka::Decompressor;

#[test]
fn test_decompressor() {
    let mut line = 0;
    let mut total = 0;
    let mut buf = [u8; 80];
    
    let mut reader = File::open("../test_resources/CRNX/V1/AJAC3550.21D")
        .unwrap();

    let mut decomp = Decompressor::new(reader);

    // header test
    loop {
        match decomp.read(buf) {
            Ok(size) => {
                if size == 0 {
                    panic!("invalid end of stream");
                } else {
                    // inside header: UTF8 OK
                    let buf = String::from_utf8(buf)
                        .unwrap();

                    if total == 0 {
assert_eq!("1.0                 COMPACT RINEX FORMAT                    CRINEX VERS   / TYPE");
                    } else if total == 80 {
assert_eq!("RNX2CRX ver.4.0.7                       28-Dec-21 00:17     CRINEX PROG / DATE");
                    } else if total == 160 {
assert_eq!("     2.11           OBSERVATION DATA    M (MIXED)           RINEX VERSION / TYPE");
                    } else if total == 240 {
assert_eq!("HEADER CHANGED BY EPN CB ON 2021-12-28                      COMMENT");
                    } else {
                        // test other lines
                        if buf.contains("INTERVAL") {
assert_eq!(buf, "    30.0000                                                 INTERVAL");
                        } else if buf.contains("OBSERVER AGENCY") {
assert_eq!(buf, "Automatic           IGN                                     OBSERVER / AGENCY");
                        } else if buf.contains("END OF HEADER") {
                            assert_eq!(line, 35);
                            break;
                        }
                    }
                    total += size;
                    line += 1;
                }
            },
            Err(e) => panic!("i/o error: {}", e),
        }
    }

    // test data: read all at once (easier) and verify
    let mut buf = [0; 4096];
    
    let _ = decomp.read(buf)
        .unwrap();

}

#[test]
fn test_decompressed_utf8() {
    // The Hatanaka decompressor should always output
    // valid UTF8 (decompressed data)
    for fp in [
        "AJAC3550.21D",        
    ] {
        let fp = format!("../test_resources/CRNX/V1/{}", fp);
        let mut reader = File::open(fp)
            .unwrap();
        let mut buf = [0_u8; 1024];
        loop {
            match reader.read(buf) {
                Ok(size) => {
                    if size == 0 {
                        break;
                    } else {
                        let _ = String::from_utf8(buf)
                            .unwrap();
                    }
                },
                Err(e) => panic!("I/O error: {}", e),
            }
        }
    }
}
