use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

use crate::hatanaka::Decompressor;

#[test]
fn test_decompressor_read() {
    let reader = File::open("../test_resources/CRNX/V1/AJAC3550.21D").unwrap();

    let mut buf = [0_u8; 128];
    let mut decomp = Decompressor::<6, _>::new(reader);

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

    //let size = decomp.read_to_string(&mut content)
    //    .unwrap();

    //let size = decomp.read_to_string(&mut content)
    //    .unwrap();

    // println!("content: \"{}\"", content);

    // // decompress header
    // while line < 35 {
    //     match decomp.read(&mut buf) {
    //         Ok(size) => {
    //             if size < 80 {
    //                 panic!("invalid read!");
    //             }
    //             let buf = String::from_utf8(buf.to_vec()).unwrap();

    //             if line == 1 {
    //                 assert_eq!(buf, "1.0                 COMPACT RINEX FORMAT                    CRINEX VERS   / TYPE");
    //             } else if line == 2 {
    //                 assert_eq!(buf, "RNX2CRX ver.4.0.7                       28-Dec-21 00:17     CRINEX PROG / DATE");
    //             } else if line == 3 {
    //                 assert_eq!(buf, "     2.11           OBSERVATION DATA    M (MIXED)           RINEX VERSION / TYPE");
    //             } else if line == 3 {
    //                 assert_eq!(
    //                     buf,
    //                     "HEADER CHANGED BY EPN CB ON 2021-12-28                      COMMENT"
    //                 );
    //             } else if line == 35 {
    //                 // test other lines
    //                 assert_eq!(buf, "                                                            END OF HEADER");
    //                 //assert_eq!(buf, "    30.0000                                                 INTERVAL");
    //                 //assert_eq!(buf, "Automatic           IGN                                     OBSERVER / AGENCY");
    //             } else if line > 35 {
    //                 panic!("header limit exceeded");
    //             }
    //             line += 1;
    //         },
    //         Err(e) => panic!("i/o error: {}", e),
    //     }
    // }
}

#[test]
fn test_decompressor_lines() {
    let mut nth = 1;
    let reader = File::open("../test_resources/CRNX/V1/AJAC3550.21D").unwrap();

    let decomp = BufReader::new(Decompressor::<6, _>::new(reader));

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
                }
                nth += 1;
            },
            Err(e) => {},
        }
        nth += 1;
    }
    assert_eq!(nth, 35);
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
