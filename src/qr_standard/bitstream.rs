// work in progress, suppressing warnings
#![allow(unused_variables)]

#[derive(Clone)]
enum BitSequence {
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

    // the bit sequence 0000
    Terminator,

    // pattern of bits that does not adhere to the standard
    Undefined,
}

#[derive(Clone)]
pub struct DataStream {
    data: Vec<BitSequence>,
    // qr_version: u8,
    // error_correction: u8,
    // valid: Option<bool>,
}

#[derive(Clone)]
pub struct BitData {
    // undecided if i want format code too
    format_code: u16,
    codewords: Vec<u8>,
}

impl DataStream {
    pub fn new() -> Self {
        DataStream { data: Vec::new() }
    }
    pub fn is_valid(&self) -> bool {
        stream_is_valid(self)
    }
    pub fn as_codewords(&self) -> BitData {
        todo!()
    }
}

impl BitData {
    pub fn codewords(&self) -> &Vec<u8> {
        &self.codewords
    }

    pub fn as_datastream(&self) -> DataStream {
        todo!()
    }
}

fn stream_is_valid(input: &DataStream) -> bool {
    todo!()
}

fn string_to_stream(input: &String) -> DataStream {
    todo!()
}

fn stream_to_bits(input: &DataStream, version: u32, error_words: u32) -> BitData {
    todo!()
}

fn bits_to_stream(input: &BitData) -> DataStream {
    todo!()
}
