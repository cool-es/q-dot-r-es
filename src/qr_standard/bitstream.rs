// work in progress, suppressing warnings
#![allow(unused_variables)]

use super::*;
use Mode::*;
use Token::*;

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
    ASCII,

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
            ASCII => 8,
            Numeric => 10,
            AlphaNum => 11,
            Kanji => 13,
        }
    }
}

impl Token {
    // maximum number of bits taken up by a token
    fn maxsize(&self) -> usize {
        match self {
            ModeAndCount(..) => 20,
            Character(mode, _) => mode.max_char_size(),
            Terminator => 4,
        }
    }
}

fn string_to_ascii(input: &str) -> Vec<Token> {
    if !input.is_ascii() {
        panic!()
    }
    let mut output: Vec<Token> = vec![ModeAndCount(ASCII, input.len() as u16)];
    for i in input.chars() {
        output.push(Character(ASCII, u16::from(i as u8)));
    }
    output
}

fn string_to_numeric(input: &str) -> Vec<Token> {
    for i in (&input).chars() {
        if !i.is_ascii_digit() {
            panic!()
        }
    }
    let mut output: Vec<Token> = vec![ModeAndCount(Numeric, input.len() as u16)];
    todo!();
    output
}

fn string_to_alphanum(input: &str) -> Vec<Token> {
    for i in (&input).chars() {
        if !super::ALPHANUMERIC_TABLE.contains(&i) {
            panic!()
        }
    }
    let mut output: Vec<Token> = vec![ModeAndCount(AlphaNum, input.len() as u16)];
    for i in input.chars() {
        output.push(Character(AlphaNum, find_alphanum(i)));
    }
    output
}
