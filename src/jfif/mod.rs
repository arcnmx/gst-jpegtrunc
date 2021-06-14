// apparently this doesn't really exist..?

pub mod markers {
	//pub use img_parts::jpeg::markers::*;
	pub const P: u8 = 0xff;
	pub const SOS: u8 = 0xda;
	pub const SOF0: u8 = 0xc0;
	pub const SOF15: u8 = SOF0 + 0xf;
	pub const RST0: u8 = 0xd0;
	pub const RST7: u8 = RST0 + 7;
	pub const SOI: u8 = 0xd8;
	pub const EOI: u8 = 0xd9;
	pub const DQT: u8 = 0xdb;
	pub const DRI: u8 = 0xdd;
	pub const APP0: u8 = 0xe0;
	pub const APP15: u8 = APP0 + 0xf;
	pub const COM: u8 = 0xfe;
}

/// non-public fn from img-parts
pub fn marker_has_length(marker: u8) -> bool {
	match marker {
		markers::RST0..=markers::RST7 => true,
		markers::APP0..=markers::APP15 => true,
		markers::SOF0..=markers::SOF15 => true,
		markers::SOS => true,
		markers::COM => true,
		markers::DQT => true,
		markers::DRI => true,
		_ => false,
	}
}
