pub mod command;
pub mod error;
pub mod utils;
pub mod value;

pub const MAGIC_NUMBER: [u8; 2] = [0x06, 0x06];
pub const VERSION: u8 = 0x01;
