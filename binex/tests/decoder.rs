use binex::prelude::{Decoder, Error, StreamElement};
use std::fs::File;

#[test]
fn mfle20190130() {
    let mut found = 0;
    let fd = File::open("../test_resources/BIN/mfle20190130.bnx").unwrap();

    let mut decoder = Decoder::new(fd);

    loop {
        match decoder.next() {
            Some(Ok(StreamElement::OpenSource(msg))) => {
                found += 1;
                println!("parsed: {:?}", msg);
            },
            Some(Ok(StreamElement::ClosedSource(element))) => {},
            Some(Err(e)) => match e {
                Error::IoError => panic!("i/o error"),
                e => {
                    println!("err={:?}", e);
                },
            },
            None => {
                break;
            },
        }
    }
    assert!(found > 0, "not a single msg decoded");
}

#[cfg(feature = "flate2")]
#[test]
fn gziped_files() {
    let mut found = 0;
    for fp in ["mfle20200105.bnx.gz", "mfle20200113.bnx.gz"] {
        let fp = format!("../test_resources/BIN/{}", fp);
        let fd = File::open(fp).unwrap();
        let mut decoder = Decoder::new_gzip(fd);

        loop {
            match decoder.next() {
                Some(Ok(StreamElement::OpenSource(msg))) => {
                    found += 1;
                    println!("parsed: {:?}", msg);
                },
                Some(Ok(StreamElement::ClosedSource(element))) => {},
                Some(Err(e)) => match e {
                    Error::IoError => panic!("i/o error"),
                    e => {
                        println!("err={:?}", e);
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
}
