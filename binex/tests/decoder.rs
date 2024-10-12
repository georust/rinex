use binex::prelude::{Decoder, Error};
use std::fs::File;

#[test]
fn mfle20190130() {
    let mut found = 0;
    let fd = File::open("../test_resources/BIN/mfle20190130.bnx").unwrap();

    let mut decoder = Decoder::new(fd);

    loop {
        match decoder.next() {
            Some(Ok(msg)) => {
                found += 1;
                println!("parsed: {:?}", msg);
            },
            Some(Err(e)) => match e {
                Error::IoError => panic!("i/o error: {}"),
                e => {
                    println!("err={}", e);
                },
            },
            None => {
                println!("EOS");
                break;
            },
        }
    }
    assert!(found > 0, "not a single msg decoded");
}

#[cfg(feature = "flate2")]
#[test]
fn gz_decoder() {
    let mut found = 0;

    for fp in [
        "mfle20190130.bnx",
        "mfle20200105.bnx.gz",
        "mfle20200113.bnx.gz",
    ] {
        let fp = format!("../test_resources/BIN/{}", fp);
        let mut fd = File::open(fp).unwrap();

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
}
