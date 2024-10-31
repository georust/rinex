use std::env;
use std::io::Write;
use std::path::Path;

fn generate_crc16_look_up_table() {
    let outdir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&outdir).join("crc16.rs");
    let mut fd = std::fs::File::create(path).unwrap();

    fd.write_all("use lazy_static::lazy_static; \n".as_bytes())
        .unwrap();

    fd.write_all("lazy_static! {\n".as_bytes()).unwrap();
    fd.write_all("static ref CRC16_TABLE : [u16; 256] = \n".as_bytes())
        .unwrap();

    // generate lut
    let polynomial = 0x1021_u16;
    let mut table = [0_u16; 256];
    for i in 0..256 {
        let mut crc = (i as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) > 0 {
                crc = (crc << 1) ^ polynomial;
            } else {
                crc = crc << 1;
            }
        }
        table[i] = crc;
        if i == 0 {
            fd.write_all(format!("[ 0x{:04X}_u16, ", crc).as_bytes())
                .unwrap();
        } else if i == 255 {
            fd.write_all(format!("0x{:04X}_u16 ];", crc).as_bytes())
                .unwrap();
        } else {
            fd.write_all(format!("0x{:04X}_u16, ", crc).as_bytes())
                .unwrap();
        }
    }
    fd.write_all("}".as_bytes()).unwrap();
}

fn main() {
    generate_crc16_look_up_table();
}
