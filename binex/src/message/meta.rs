#[derive(Debug, Copy, Clone, Default, PartialEq)]
/// [Meta] data that describes the core structure of each message.
pub struct Meta {
    /// Whether this message is reversed or not
    pub reversed: bool,
    /// Whether this message uses enhanced CRC or not.
    /// Enhanced CRC robusties the transmissions a lot.
    pub enhanced_crc: bool,
    /// Endianess used during encoding process.
    pub big_endian: bool,
}

impl Meta {
    // Forward stream +LE +standard
    const FWD_SYNC_LE_STANDARD_CRC: u8 = 0xc2;
    // Forward stream +BE +standard
    const FWD_SYNC_BE_STANDARD_CRC: u8 = 0xe2;
    // Forward stream +LE +enhanced
    const FWD_SYNC_LE_ENHANCED_CRC: u8 = 0xc8;
    // Forward stream +BE +enhanced
    const FWD_SYNC_BE_ENHANCED_CRC: u8 = 0xe8;

    // Reversed stream +LE +standard
    const REV_SYNC_LE_STANDARD_CRC: u8 = 0xd2;
    // Reversed stream +BE +standard
    const REV_SYNC_BE_STANDARD_CRC: u8 = 0xf2;
    // Reversed stream +LE +enhanced
    const REV_SYNC_LE_ENHANCED_CRC: u8 = 0xd8;
    // Reversed stream +BE +enhanced
    const REV_SYNC_BE_ENHANCED_CRC: u8 = 0xf8;
}

impl Meta {
    /// Generates correct Sync byte to initiate this [Meta] stream
    pub(crate) fn sync_byte(&self) -> u8 {
        if self.reversed {
            if self.big_endian {
                if self.enhanced_crc {
                    Self::REV_SYNC_BE_ENHANCED_CRC
                } else {
                    Self::REV_SYNC_BE_STANDARD_CRC
                }
            } else if self.enhanced_crc {
                Self::REV_SYNC_LE_ENHANCED_CRC
            } else {
                Self::REV_SYNC_LE_STANDARD_CRC
            }
        } else if self.big_endian {
            if self.enhanced_crc {
                Self::FWD_SYNC_BE_ENHANCED_CRC
            } else {
                Self::FWD_SYNC_BE_STANDARD_CRC
            }
        } else if self.enhanced_crc {
            Self::FWD_SYNC_LE_ENHANCED_CRC
        } else {
            Self::FWD_SYNC_LE_STANDARD_CRC
        }
    }
    /// Locate SYNC byte in provided buffer
    /// ## Returns
    /// - [Meta]
    /// - sync byte offset
    pub(crate) fn find_and_parse(buf: &[u8], size: usize) -> Option<(Self, usize)> {
        let mut meta = Meta::default();
        for i in 0..size {
            if buf[i] == Self::FWD_SYNC_BE_ENHANCED_CRC {
                meta.enhanced_crc = true;
                meta.big_endian = true;
                return Some((meta, i));
            } else if buf[i] == Self::FWD_SYNC_LE_ENHANCED_CRC {
                meta.enhanced_crc = true;
                return Some((meta, i));
            } else if buf[i] == Self::FWD_SYNC_BE_STANDARD_CRC {
                meta.big_endian = true;
                return Some((meta, i));
            } else if buf[i] == Self::FWD_SYNC_LE_STANDARD_CRC {
                return Some((meta, i));
            } else if buf[i] == Self::REV_SYNC_BE_ENHANCED_CRC {
                meta.reversed = true;
                meta.big_endian = true;
                meta.enhanced_crc = true;
                return Some((meta, i));
            } else if buf[i] == Self::REV_SYNC_LE_ENHANCED_CRC {
                meta.reversed = true;
                meta.enhanced_crc = true;
                return Some((meta, i));
            } else if buf[i] == Self::REV_SYNC_BE_STANDARD_CRC {
                meta.reversed = true;
                meta.big_endian = true;
                return Some((meta, i));
            } else if buf[i] == Self::REV_SYNC_LE_STANDARD_CRC {
                meta.reversed = true;
                return Some((meta, i));
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::Meta;
    #[test]
    fn forge_sync_byte() {
        for (meta, expected) in [
            (
                Meta {
                    reversed: false,
                    big_endian: true,
                    enhanced_crc: true,
                },
                Meta::FWD_SYNC_BE_ENHANCED_CRC,
            ),
            (
                Meta {
                    reversed: false,
                    big_endian: false,
                    enhanced_crc: true,
                },
                Meta::FWD_SYNC_LE_ENHANCED_CRC,
            ),
            (
                Meta {
                    reversed: false,
                    big_endian: true,
                    enhanced_crc: false,
                },
                Meta::FWD_SYNC_BE_STANDARD_CRC,
            ),
            (
                Meta {
                    reversed: false,
                    big_endian: false,
                    enhanced_crc: false,
                },
                Meta::FWD_SYNC_LE_STANDARD_CRC,
            ),
            (
                Meta {
                    reversed: true,
                    big_endian: false,
                    enhanced_crc: true,
                },
                Meta::REV_SYNC_LE_ENHANCED_CRC,
            ),
            (
                Meta {
                    reversed: true,
                    big_endian: false,
                    enhanced_crc: false,
                },
                Meta::REV_SYNC_LE_STANDARD_CRC,
            ),
            (
                Meta {
                    reversed: true,
                    big_endian: true,
                    enhanced_crc: true,
                },
                Meta::REV_SYNC_BE_ENHANCED_CRC,
            ),
            (
                Meta {
                    reversed: true,
                    big_endian: true,
                    enhanced_crc: false,
                },
                Meta::REV_SYNC_BE_STANDARD_CRC,
            ),
        ] {
            assert_eq!(meta.sync_byte(), expected);
        }
    }

    #[test]
    fn test_sync_byte_matcher() {
        for i in 0..256 {
            let val8 = i as u8;
            let buf = [0, val8];

            let meta = Meta::find_and_parse(&buf, 2);

            match val8 {
                0xc2 => {
                    // FWD +LE +STANDARD
                    let (meta, size) = meta.expect("did not detect sync byte");
                    assert_eq!(size, 1);
                    assert!(!meta.reversed);
                    assert!(!meta.big_endian);
                    assert!(!meta.enhanced_crc);
                },
                0xe2 => {
                    // FWD +BE +STANDARD
                    let (meta, size) = meta.expect("did not detect sync byte");
                    assert_eq!(size, 1);
                    assert!(!meta.reversed);
                    assert!(meta.big_endian);
                    assert!(!meta.enhanced_crc);
                },
                0xc8 => {
                    // FWD +LE +ENHANCED
                    let (meta, size) = meta.expect("did not detect sync byte");
                    assert_eq!(size, 1);
                    assert!(!meta.reversed);
                    assert!(!meta.big_endian);
                    assert!(meta.enhanced_crc);
                },
                0xe8 => {
                    // FWD +BE +ENHANCED
                    let (meta, size) = meta.expect("did not detect sync byte");
                    assert_eq!(size, 1);
                    assert!(!meta.reversed);
                    assert!(meta.big_endian);
                    assert!(meta.enhanced_crc);
                },
                0xd2 => {
                    // REV +LE +STANDARD
                    let (meta, size) = meta.expect("did not detect sync byte");
                    assert_eq!(size, 1);
                    assert!(meta.reversed);
                    assert!(!meta.big_endian);
                    assert!(!meta.enhanced_crc);
                },
                0xf2 => {
                    // REV +BE + STANDARD
                    let (meta, size) = meta.expect("did not detect sync byte");
                    assert_eq!(size, 1);
                    assert!(meta.reversed);
                    assert!(meta.big_endian);
                    assert!(!meta.enhanced_crc);
                },
                0xd8 => {
                    // REV +LE +ENHANCED
                    let (meta, size) = meta.expect("did not detect sync byte");
                    assert_eq!(size, 1);
                    assert!(meta.reversed);
                    assert!(!meta.big_endian);
                    assert!(meta.enhanced_crc);
                },
                0xf8 => {
                    // REV +BE +ENHANCED
                    let (meta, size) = meta.expect("did not detect sync byte");
                    assert_eq!(size, 1);
                    assert!(meta.reversed);
                    assert!(meta.big_endian);
                    assert!(meta.enhanced_crc);
                },
                _ => {
                    assert!(meta.is_none(), "found invalid sync byte for 0x{:01x}", val8);
                },
            }
        }
    }
}
