use super::*;

enum DataChunk {
    // not implementing ECI at this time

    // mode indicator
    // 4 bits
    Mode(u8),

    // character count
    // 8 bits (i believe!!)
    Count(u8),

    // ascii/shift-jis byte
    // 1 character / 8 bits
    Byte(u8),

    // base-10 numbers
    // 3 numbers / 10 bits
    // i'm not sure if the bit length is constant
    Numeric(u16),

    // alphanumeric + 9 symbols (see notes)
    // 2 characters / 11 bits
    AlphaNum(u16),

    // 1 character / 13 bits
    Kanji(u16),

    Terminator,
}

// this is also a possible solution but it's doing too much at once
/* enum DataChunk2 {
    Byte { count: u32, data: Vec<u8> },
    Numeric { count: u32, data: Vec<u8> },
    AlphaNum { count: u32, data: Vec<u8> },
    Terminator,
} */

#[derive(Clone)]
pub struct DataStream {
    data: Vec<DataChunk>,
    qr_version: u8,
    error_correction: u8,
    valid: Option<bool>,
}

#[derive(Clone)]
pub struct BitData {
    // undecided if i want format code too
    format_code: u16,
    codewords: Vec<u8>,
}

impl DataStream {
    pub fn is_valid(&self) -> bool {
        self.valid == Some(true)
    }
    pub fn validate(&mut self) {
        // set a value for self.valid
        todo!();
    }
}
