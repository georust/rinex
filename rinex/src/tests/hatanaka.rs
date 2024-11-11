use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

use crate::hatanaka::Decompressor;

#[test]
fn test_decompressor_read() {
    let reader = File::open("../test_resources/CRNX/V1/AJAC3550.21D").unwrap();

    let mut buf = [0_u8; 128];
    let mut decomp = Decompressor::new(reader);

    let size = decomp.read(&mut buf).unwrap();
    // assert_eq!(size, 81);

    let string = String::from_utf8(buf[..size].to_vec()).unwrap();
    assert_eq!(
        string,
        "     2.11           OBSERVATION DATA    M (MIXED)           RINEX VERSION / TYPE\n"
    );

    buf = [0_u8; 128];
    let size = decomp.read(&mut buf).unwrap();
    // assert_eq!(size, 81);

    let string = String::from_utf8(buf[..size].to_vec()).unwrap();
    assert_eq!(
        string,
        "HEADER CHANGED BY EPN CB ON 2021-12-28                      COMMENT\n"
    );

    loop {
        match decomp.read(&mut buf) {
            Ok(0) => break,
            Ok(size) => {
                let _ = String::from_utf8(buf[..size].to_vec()).unwrap();
            },
            Err(e) => {},
        }
    }
}

#[test]
fn test_decompressor_lines() {
    let mut nth = 1;
    let reader = File::open("../test_resources/CRNX/V1/AJAC3550.21D").unwrap();

    let decomp = BufReader::new(Decompressor::new(reader));

    let mut lines = decomp.lines();

    while let Some(line) = lines.next() {
        match line {
            Ok(line) => {
                if nth == 1 {
                    assert_eq!(line, "     2.11           OBSERVATION DATA    M (MIXED)           RINEX VERSION / TYPE");
                } else if nth == 2 {
                    assert_eq!(
                        line,
                        "HEADER CHANGED BY EPN CB ON 2021-12-28                      COMMENT"
                    );
                } else if nth == 3 {
                    assert_eq!(
                        line,
                        "TO BE CONFORM WITH THE INFORMATION IN                       COMMENT"
                    );
                } else if nth == 4 {
                    assert_eq!(
                        line,
                        "ftp://epncb.oma.be/pub/station/log/ajac.log                 COMMENT"
                    );
                } else if nth == 5 {
                    assert_eq!(
                        line,
                        "                                                            COMMENT"
                    );
                } else if nth == 6 {
                    assert_eq!(line, "teqc  2019Feb25     IGN-RGP             20211222 00:07:07UTCPGM / RUN BY / DATE");
                } else if nth == 7 {
                    assert_eq!(
                        line,
                        "Linux 2.6.32-573.12.1.x86_64|x86_64|gcc|Linux 64|=+         COMMENT"
                    );
                } else if nth == 12 {
                    assert_eq!(
                        line,
                        "BIT 2 OF LLI FLAGS DATA COLLECTED UNDER A/S CONDITION       COMMENT"
                    );
                } else if nth == 13 {
                    assert_eq!(
                        line,
                        "AJAC                                                        MARKER NAME"
                    );
                } else if nth == 32 {
                    assert_eq!(line,"  2021    12    21     0     0    0.0000000     GPS         TIME OF FIRST OBS");
                } else if nth == 33 {
                    assert_eq!(
                        line,
                        "                                                            END OF HEADER"
                    );
                } else {
                }
                nth += 1;
            },
            Err(e) => {
                panic!("unexpected error: {}", e);
            },
        }
    }
    assert_eq!(nth, 92);
}

#[test]
fn test_decompressed_utf8() {
    // The Hatanaka decompressor should always output
    // valid UTF8 (decompressed data)
    for fp in ["AJAC3550.21D"] {
        let fp = format!("../test_resources/CRNX/V1/{}", fp);
        let mut reader = File::open(fp).unwrap();
        let mut buf = [0_u8; 1024];
        loop {
            match reader.read(&mut buf) {
                Ok(size) => {
                    if size == 0 {
                        break;
                    } else {
                        let _ = String::from_utf8(buf.to_vec()).unwrap();
                    }
                },
                Err(e) => panic!("I/O error: {}", e),
            }
        }
    }
}
