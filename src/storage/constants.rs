pub const MAGIC_NUMBER: &[u8] = b"REDIS";

pub mod opcode {
    pub const META: u8 = 0xFA;
    pub const START_DB: u8 = 0xFE;
    pub const RESIZE_DB: u8 = 0xFB;
    pub const KEY_WITH_EXPIRATION_MS: u8 = 0xFC;
    pub const KEY_WITH_EXPIRATION_SEC: u8 = 0xFD;
    pub const EOF : u8 = 255;
}

pub mod blob_encoding {
    pub const INT8 : u32 = 0;
    pub const INT16 : u32 = 1;
    pub const INT32 : u32 = 2;
    pub const LZF : u32 = 3;
}

pub mod encoding_type {
    pub const STRING : u8 = 0;
    // pub const LIST : u8 = 1;
    // pub const SET : u8 = 2;
    // pub const ZSET : u8 = 3;
    // pub const HASH : u8 = 4;
    // pub const HASH_ZIPMAP : u8 = 9;
    // pub const LIST_ZIPLIST : u8 = 10;
    // pub const SET_INTSET : u8 = 11;
    // pub const ZSET_ZIPLIST : u8 = 12;
    // pub const HASH_ZIPLIST : u8 = 13;
    // pub const LIST_QUICKLIST : u8 = 14;
}