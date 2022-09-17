use bitflags::bitflags;

bitflags! {
	/// GNSS / GPS orbit health indication
	#[derive(Default)]
	#[cfg_attr(feature = "serde", derive(Serialize))]
	pub struct Health: u32 {
		const L1_HEALTHY = 0x01;
		const L2_HEALTHY = 0x02;
		const L5_HEALTHY = 0x04;
	}
	
	/// IRNSS orbit health indication
	#[derive(Default)]
	#[cfg_attr(feature = "serde", derive(Serialize))]
	pub struct IrnssHealth: u32 {
		const UNKNOWN = 0x01;
	}
	
	/// SBAS/GEO orbit health indication
	#[derive(Default)]
	#[cfg_attr(feature = "serde", derive(Serialize))]
	pub struct GeoHealth: u32 {
		const RESERVED = 0x08;
	}
	
	/// GAL orbit health indication
	#[derive(Default)]
	#[cfg_attr(feature = "serde", derive(Serialize))]
	pub struct GalHealth: u32 {
		const E1B_DVS = 0x01;
		const E1B_HS0 = 0x02;
		const E1B_HS1 = 0x04;
		const E5A_DVS = 0x08;
		const E5A_HS0 = 0x10;
		const E5A_HS1 = 0x20;
		const E5B_HS0 = 0x40;
		const E5B_HS1 = 0x80;
	}
}
