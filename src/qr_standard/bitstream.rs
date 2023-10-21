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

#[derive(Clone, PartialEq)]
pub(crate) enum Mode {
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
// #[derive(Clone)]
// struct MarkedString {
//     mode: Mode,
//     string: String,
// }

// level 3
#[derive(Clone)]
pub(super) enum Token {
    // mode and character count indicators,
    // baked into one
    ModeAndCount(Mode, u16),

    // one character, which can vary in length
    // between 4 and 13 bits
    // mode, bit length, bit value
    // the mode field might be superfluous...
    Character(Mode, usize, u16),

    // the bit sequence 0000
    Terminator,
}

fn string_to_ascii(input: &str) -> Vec<Token> {
    if !input.is_ascii() {
        panic!("not ascii")
    }
    let mut output: Vec<Token> = vec![ModeAndCount(ASCII, input.len() as u16)];
    for i in input.chars() {
        output.push(Character(ASCII, 8, u16::from(i as u8)));
    }
    output
}

fn string_to_numeric(input: &str) -> Vec<Token> {
    for i in (&input).chars() {
        if !i.is_ascii_digit() {
            panic!("not numeric")
        }
    }
    let mut output: Vec<Token> = vec![ModeAndCount(Numeric, input.len() as u16)];

    for i in input
        .chars()
        .map(|x| x.to_digit(10).unwrap() as u16)
        .collect::<Vec<u16>>()
        .chunks(3)
    {
        if i.len() == 3 {
            output.push(Character(AlphaNum, 10, i[0] * 100 + i[1] * 10 + i[2]));
        } else if i.len() == 2 {
            output.push(Character(AlphaNum, 7, i[0] * 10 + i[1]));
        } else {
            output.push(Character(AlphaNum, 4, i[0]));
        }
    }
    output
}

fn string_to_alphanum(input: &str) -> Vec<Token> {
    let mut output: Vec<Token> = vec![ModeAndCount(AlphaNum, input.len() as u16)];
    for i in input
        .chars()
        .map(|x| find_alphanum(x))
        .collect::<Vec<u16>>()
        .chunks(2)
    {
        if i.len() == 2 {
            output.push(Character(AlphaNum, 11, i[0] * 45 + i[1]));
        } else {
            output.push(Character(AlphaNum, 6, i[0]));
        }
    }
    output
}

// KISS
fn push_token_to_badstream(stream: &mut Badstream, token: Token, version: u32) {
    match token {
        ModeAndCount(mode, count) => {
            let a: (usize, &str) = match mode {
                Numeric => (0, "0001"),
                AlphaNum => (1, "0010"),
                ASCII => (2, "0100"),
                Kanji => (3, "1000"),
            };
            let b = match version {
                1..=9 => 0,
                10..=26 => 1,
                27..=40 => 2,
                _ => panic!(),
            };

            // number of bits in char count indicator - see pg. 24
            let width: usize = [[10, 9, 8, 8], [12, 11, 16, 10], [14, 13, 16, 12]][b][a.0];
            let string = format!("{:016b}", count);

            push_bits(a.1, stream);
            push_bits(&string[(16 - width)..], stream);
        }
        Character(_, width, address) => {
            let string = format!("{:016b}", address);
            push_bits(&string[(16 - width)..], stream);
        }
        Terminator => {
            push_bits("0000", stream);
        }
    }
}

pub(super) fn make_token_stream(input: &[(Mode, &str)]) -> Vec<Token> {
    let mut stream: Vec<Token> = Vec::new();
    for (mode, data) in input {
        stream.extend(match mode {
            Numeric => string_to_numeric(&data),
            AlphaNum => string_to_alphanum(&data),
            ASCII => string_to_ascii(&data),
            _ => panic!(),
        });
    }
    stream.push(Terminator);

    stream
}

pub(super) fn tokens_to_badstream(stream: Vec<Token>, version: u32) -> Badstream {
    let mut output: Badstream = Vec::new();
    for token in stream {
        push_token_to_badstream(&mut output, token, version);
    }
    output
}

pub(crate) fn invoke_modes(input: &[(Mode, &str)], version: u32) -> Badstream {
    tokens_to_badstream(make_token_stream(input), version)
}

// no. of bits independent of version + char count indicators:
// numeric, alphanumeric, ascii, kanji
type Overhead = (usize, [usize; 4]);

fn bit_overhead_template(data: &Vec<Token>) -> Overhead {
    let mut bit_sum = 0;
    let mut count_indicators = [0; 4];

    for i in data {
        match i {
            ModeAndCount(mode, _) => {
                let m = match mode {
                    Numeric => 0,
                    AlphaNum => 1,
                    ASCII => 2,
                    Kanji => 3,
                };

                bit_sum += 4;
                count_indicators[m] += 1;
            }
            Character(_, length, _) => bit_sum += *length,
            Terminator => bit_sum += 4,
        }
    }
    (bit_sum, count_indicators)
}

fn compute_bit_overhead(overhead: Overhead, version: u32) -> usize {
    let table = match version {
        // no. of bits in char count indicator per version
        1..=9 => [10, 9, 8, 8],
        10..=26 => [12, 11, 16, 10],
        27..=40 => [14, 13, 16, 12],
        _ => panic!(),
    };
    let (mut sum, indicators) = overhead;
    for m in 0..=3 {
        sum += table[m] * indicators[m];
    }
    sum
}

pub(super) fn bit_overhead(data: &Vec<Token>, version: u32) -> usize {
    compute_bit_overhead(bit_overhead_template(data), version)
}

pub(super) fn find_best_version(data: &Vec<Token>, level: u8) -> u32 {
    assert!((0..=3).contains(&level));
    let table = DATA_CODEWORDS[level as usize];
    let overhead = bit_overhead_template(data);

    for version in 1..=40 {
        let diff =
            // compute_bit_overhead(overhead, version).checked_sub(8 * table[version as usize - 1]);
            (8 * table[version as usize - 1]).checked_sub(compute_bit_overhead(overhead, version));
        if let Some(x) = diff {
            if x == 0 || x > 7 {
                return version;
            }
        }
    }

    panic!(
        "no qr code of level {} fits this message",
        "LMQH".chars().collect::<Vec<_>>()[level as usize]
    )
}

#[test]
fn bit_overhead_good() {
    let input = &[
        (0, "14"),
        (2, "hello!\n\n"),
        (1, "HOHO..."),
        (0, "123123"),
        (1, "OHH"),
    ];
    for version in 10..=40 {
        // copy of invoke_modes, with changes
        let mut stream: Vec<Token> = Vec::new();
        for (mode, data) in input {
            stream.extend(match mode {
                0 => string_to_numeric(&data),
                1 => string_to_alphanum(&data),
                2 => string_to_ascii(&data),
                _ => panic!(),
            });
        }
        stream.push(Terminator);

        let check_len = bit_overhead(&stream, version);

        let mut output: Badstream = Vec::new();
        for token in stream {
            push_token_to_badstream(&mut output, token, version);
        }

        assert!(check_len == output.len(), "bit overhead calculation");
    }
}
