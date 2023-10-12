// work in progress, suppressing warnings
#![allow(unused_variables)]

/*
something that's really complicated is deciding what level of complexity/abstraction i want to tackle this problem at. there are really 4 different levels:
1. raw input string
2. raw substrings with mode indicators added
3. (a vector of) individual tokens
4. bits
and i was stuck choosing between 2 and 3, where either option would make it really complicated to skip over the missing step. so i chose to do both
*/

#[derive(Clone)]
enum Mode {
    // not implementing ECI at this time

    // ascii/shift-jis byte
    // 1 character / 8 bits
    Byte,

    // base-10 numbers
    // 10, 7 or 4 bits
    Numeric,

    // alphanumeric + 9 symbols (see notes)
    // 2 characters / 11 bits
    AlphaNum,

    // 1 character / 13 bits
    Kanji,
}

// level 2
#[derive(Clone)]
struct MarkedString {
    mode: Mode,
    string: String,
}

// level 3
#[derive(Clone)]
enum Token {
    // mode and character count indicators,
    // baked into one
    ModeAndCount(Mode, u16),

    // one character, which can vary in length
    // between 4 and 13 bits
    Character(Mode, u16),

    // the bit sequence 0000
    Terminator,
}

impl Mode {
    // maximum number of bits taken up by a character
    fn max_char_size(&self) -> usize {
        match self {
            Self::Byte => 8,
            Self::Numeric => 10,
            Self::AlphaNum => 11,
            Self::Kanji => 13,
        }
    }
}

impl Token {
    // maximum number of bits taken up by a token
    fn maxsize(&self) -> usize {
        match self {
            Self::ModeAndCount(..) => 20,
            Self::Character(mode, _) => mode.max_char_size(),
            Self::Terminator => 4,
        }
    }
}
