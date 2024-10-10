pub struct Constants {}

impl Constants {
    /// Forward Little Endian stream with standard CRC
    pub const FWDSYNC_LE_STANDARD_CRC: u8 = 0xC2;

    /// Forward Big Endian stream with standard CRC
    pub const FWDSYNC_BE_STANDARD_CRC: u8 = 0xE2;

    /// Forward Little Endian stream with enhanced CRC
    pub const FWDSYNC_LE_ENHANCED_CRC: u8 = 0xC8;

    /// Forward Big Endian stream with enhanced CRC
    pub const FWDSYNC_BE_ENHANCED_CRC: u8 = 0xE8;

    /// Rerversed Little Endian stream with standard CRC
    pub const REVSYNC_LE_STANDARD_CRC: u8 = 0xD2;

    /// Rerversed Big Endian stream with standard CRC
    pub const REVSYNC_BE_STANDARD_CRC: u8 = 0xF2;

    /// Rerversed Little Endian stream with enhanced CRC
    pub const REVSYNC_LE_ENHANCED_CRC: u8 = 0xD8;

    /// Rerversed Big Endian stream with enhanced CRC
    pub const REVSYNC_BE_ENHANCED_CRC: u8 = 0xF8;

    /// Keep going byte mask in the BNXI algorithm,
    /// as per [https://www.unavco.org/data/gps-gnss/data-formats/binex/conventions.html/#ubnxi_details]
    pub const BNXI_KEEP_GOING_MASK: u8 = 0x80;

    /// Data byte mask in the BNXI algorithm,
    /// as per [https://www.unavco.org/data/gps-gnss/data-formats/binex/conventions.html/#ubnxi_details]
    pub const BNXI_BYTE_MASK: u8 = 0x7f;
}
