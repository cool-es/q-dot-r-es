// work in progress, suppressing warnings
#![allow(unused_variables)]

#[derive(Clone)]
enum Token {
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
}

#[derive(Clone)]
pub struct DataStream {
    data: Vec<Token>,
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
}

impl BitData {
    pub fn codewords(&self) -> &Vec<u8> {
        &self.codewords
    }
}

impl Token {
    // number of bits taken up by a token
    fn size(&self) -> usize {
        match self {
            // not completely sure about this!
            Self::Mode(_) => 4,
            Self::Count(_) => 8,
            Self::Byte(_) => 8,
            Self::Numeric(_) => 10,
            Self::AlphaNum(_) => 11,
            Self::Kanji(_) => 13,
            Self::Terminator => 4,
            // _ => panic!(),
        }
    }
}

// untested
fn set_bit(input: &mut BitData, index: usize, state: bool) {
    let (n, i) = (index / 8, index % 8);
    if state {
        // set a 1 (bitwise 'or' w/ 1)
        input.codewords[n] |= 1 << i;
    } else {
        //set a 0 (bitwise 'and' w/ 0)
        input.codewords[n] &= !(1 << i);
    }
}
