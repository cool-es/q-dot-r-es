use super::*;
use Mode::*;
use Token::*;

/// The algorithm for size-optimal mode switching.
pub mod search;

/*
something that's really complicated is deciding what level of complexity/abstraction i want to tackle this problem at. there are really 4 different levels:
1. raw input string
2. raw substrings with mode indicators added
3. (a vector of) individual tokens
4. bits
and i was stuck choosing between 2 and 3, where either option would make it really complicated to skip over the missing step. so i chose to do both
*/

/// The different sets a character can be part of.
///
/// The set of kanji characters are isolated from the
/// rest, but every numeric character is also an
/// alphanumeric character, and every alphanumeric
/// character is an ASCII character. The reason for
/// characters to be contained in multiple modes
/// like this is that it allows for more efficient
/// data compression.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub(crate) enum Mode {
    /// Base-10 digits: 0 to 9.
    ///
    /// A numeric [token character](Token::Character) contains at most 3 digits, which
    /// takes up 10 bits. If the total number of digits encoded
    /// isn't divisible by 3, the final token will contain
    /// either two digits (7 bits) or one (4 bits).
    Numeric,

    /// Digits 0 to 9, capital letters A to Z, and nine special
    /// characters: ` `, `$`, `%`, `*`, `+`, `-`, `.`, `/`, and `:`.
    ///
    /// An alphanumeric [token character](Token::Character) contains at most two characters,
    /// which takes up 11 bits. If the total number of alphanumerics
    /// encoded is odd, the final token will just contain a single
    /// character (6 bits).
    AlphaNum,

    /// The full ASCII character set.
    ///
    /// These characters aren't compressed in any way -- they are written
    /// to the QR code as-is. Therefore, each ASCII [token character](Token::Character)
    /// is exactly one byte.
    ASCII,

    /// Currently unimplemented.
    ///
    /// One kanji [token character](Token::Character) fits 1 character in 13 bits.
    #[allow(dead_code)]
    Kanji,
}

impl Mode {
    /// The three ASCII subsets, ordered by inclusion.
    pub(crate) const LIST: [Self; 3] = [ASCII, AlphaNum, Numeric];
}

// level 3
#[derive(Clone)]
pub(super) enum Token {
    /// mode and character count indicators,
    /// baked into one.
    ModeAndCount(Mode, u16),

    /// one character, which can vary in length
    /// between 4 and 13 bits.
    ///
    /// fields are mode, bit length, bit value.
    /// the mode field might be superfluous...
    Character(Mode, usize, u16),

    /// the bit sequence `0000`
    Terminator,
}

fn string_to_ascii(input: &str) -> Vec<Token> {
    assert!(input.is_ascii(), "invalid ascii input!");
    let mut output: Vec<Token> = vec![ModeAndCount(ASCII, input.len() as u16)];
    for i in input.chars() {
        output.push(Character(ASCII, 8, u16::from(i as u8)));
    }
    output
}

fn string_to_numeric(input: &str) -> Vec<Token> {
    let mut output: Vec<Token> = vec![ModeAndCount(Numeric, input.len() as u16)];

    for i in input
        .chars()
        .map(|x| x.to_digit(10).expect("invalid numeric input!") as u16)
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
        .map(|x| {
            ALPHANUM_SET
                .find(x)
                .map(|x| x as u16)
                .expect("invalid alphanumeric input!")
        })
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

/// Convert a `Token` character into its equivalent bit sequence.
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

/// Stitch a vector of labeled strings into a vector of `Token` characters.
pub(super) fn make_token_stream(input: Vec<(Mode, String)>) -> Vec<Token> {
    let mut stream: Vec<Token> = Vec::new();
    for (mode, data) in input {
        stream.extend(match mode {
            Numeric => string_to_numeric(&data),
            AlphaNum => string_to_alphanum(&data),
            ASCII => string_to_ascii(&data),
            _ => panic!("unsupported mode"),
        });
    }
    stream.push(Terminator);

    stream
}

/// Convert a vector of tokens into a single stream of bits.
pub(super) fn tokens_to_badstream(stream: Vec<Token>, version: u32) -> Badstream {
    let mut output: Badstream = Vec::new();
    for token in stream {
        push_token_to_badstream(&mut output, token, version);
    }
    output
}

/// A template to calculate the bit size of a series of tokens.
///
/// ```
/// let (guaranteed_bits, [numeric_cc, alphanumeric_cc, ascii_cc, kanji_cc]): Overhead;
/// ```
/// Holds a count of the number of guaranteed bits in the encoded
/// message, as well as the number of numeric, alphanumeric,
/// ASCII, and kanji character count indicators, respectively.
/// As the character count markers vary in size depending on
/// the QR code's version (see [cc_indicator_bit_size]), the
/// exact size of a message can't be known in advance.
///
/// For example, an ASCII string with 1 character in it contains
/// 4 bits for a mode marker, 8 bits for the character, 4 bits
/// for the terminator, and a single ASCII character count indicator:
/// ```
/// let data_vec = vec![(ASCII, "a".to_string())];
/// let token_vec = make_token_stream(data_vec);
/// let template = bit_overhead_template(&token_vec);
///
/// assert_eq!(template, (16, [0, 0, 1, 0]));
/// ```
type Overhead = (usize, [usize; 4]);

fn bit_overhead_template(data: &Vec<Token>) -> Overhead {
    let mut bit_sum = 0;
    let mut count_indicators = [0; 4];

    for i in data {
        match i {
            ModeAndCount(mode, _) => {
                bit_sum += 4;
                count_indicators[match mode {
                    Numeric => 0,
                    AlphaNum => 1,
                    ASCII => 2,
                    Kanji => 3,
                }] += 1;
            }
            Character(_, length, _) => bit_sum += *length,
            Terminator => bit_sum += 4,
        }
    }
    (bit_sum, count_indicators)
}

fn compute_bit_overhead(overhead: Overhead, version: u32) -> usize {
    let table = CC_INDICATOR_BITS[match version {
        // no. of bits in char count indicator per version
        1..=9 => 0,
        10..=26 => 1,
        27..=40 => 2,
        _ => panic!(),
    }];
    let (mut sum, indicators) = overhead;
    for m in 0..=3 {
        sum += table[m] * indicators[m];
    }
    sum
}

/// Finds the smallest QR code version that fits a token stream.
///
/// There are two major caveats that make this process more complex:
/// * The size of the character count indicators, which are encoded
/// in the bit sequence, get larger with higher versions (refer to
/// [cc_indicator_bit_size]).
/// * Empirically, a QR code with less than a full codeword/byte of
/// padding to spare (e.g., 5 bits) will not be scannable. The QR
/// standards document does not explain why. This function circumvents
/// the issue by requiring that codes either fit with either exactly
/// 0 bits to spare, or at least a full byte.
pub(super) fn find_best_version(data: &Vec<Token>, level: u8) -> Result<u32, String> {
    assert!(
        (0..=3).contains(&level),
        "invalid error correction level \"{}\" selected",
        level
    );
    let table = DATA_CODEWORDS[level as usize];
    let overhead = bit_overhead_template(data);

    for version in 1..=40 {
        // total number of bits that fit in the qr code, minus bit length of message
        let diff =
            (8 * table[version as usize - 1]).checked_sub(compute_bit_overhead(overhead, version));
        if let Some(x) = diff {
            // check to see that the bitstream either fits perfectly,
            // or has at least one byte to spare
            if x == 0 || x > 7 {
                return Ok(version);
            }
        }
    }

    Err(format!(
        "no qr code of level {} fits this message",
        b"LMQH"[level as usize] as char
    ))
}

/// Returns the smallest mode subset of an ASCII character.
fn char_status(x: char) -> Option<Mode> {
    Some(if x.is_ascii_digit() {
        // ascii, alphanumeric and numeric
        Numeric
    } else if ALPHANUM_SET.contains(x) {
        // ascii and alphanumeric
        AlphaNum
    } else if x.is_ascii() {
        // only ascii
        ASCII
    } else {
        return None;
    })
}
