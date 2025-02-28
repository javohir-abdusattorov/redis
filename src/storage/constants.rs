pub const OPCODE_START_DB: u8 = 0xFE;

#[allow(dead_code)]
pub const OPCODE_EXPIRETIME_MS: u8 = 0xFC;

#[allow(dead_code)]
pub const OPCODE_EXPIRETIME_S: u8 = 0xFD;
pub const OPCODE_META: u8 = 0xFA;

#[allow(dead_code)]
pub const OPCODE_SIZE: u8 = 0xFB;
pub const OPCODE_EOF: u8 = 0xFF;
#[allow(dead_code)]
pub const OPCODE_STRING: u8 = 0x00;
#[allow(dead_code)]
pub const OPCODE_LIST: u8 = 0x01;
#[allow(dead_code)]
pub const OPCODE_HASH: u8 = 0x04;
pub const MAGIC_NUMBER: &[u8] = b"REDIS";