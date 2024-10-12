use binex::prelude::{Decoder, Error};
use std::fs::File;

#[test]
fn parse_cres_2008() {
    let mut found = 0;

    let mut fd = File::open("../test_resources/BIN/cres_20080526.bin").unwrap();

    let mut decoder = Decoder::new(fd);

    while let Some(ret) = decoder.next() {
        match ret {
            Ok(msg) => {
                found += 1;
            },
            Err(e) => match e {
                Error::IoError(e) => panic!("i/o error: {}", e),
                e => panic!("other error: {}", e),
            },
        }
    }

    assert!(found > 0, "not a single msg decoded");
}
