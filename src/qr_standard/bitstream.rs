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

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(crate) enum Mode {
    // not implementing ECI at this time

    // base-10 numbers
    // 10, 7 or 4 bits
    Numeric,

    // alphanumeric + 9 symbols (see notes)
    // 2 characters / 11 bits
    AlphaNum,

    // ascii/shift-jis byte
    // 1 character / 8 bits
    ASCII,

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
    for i in input.chars() {
        if !i.is_ascii_digit() {
            panic!("character \"{}\" is not numeric", i)
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
        .map(|x| {
            find_alphanum(x).unwrap_or_else(|| panic!("character \"{}\" is not alphanumeric", x))
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

pub(super) fn tokens_to_badstream(stream: Vec<Token>, version: u32) -> Badstream {
    let mut output: Badstream = Vec::new();
    for token in stream {
        push_token_to_badstream(&mut output, token, version);
    }
    output
}

pub(crate) fn invoke_modes(input: Vec<(Mode, String)>, version: u32) -> Badstream {
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

pub fn compute_bit_hypothetical() {
    let modes = [ASCII, AlphaNum, Numeric];
    for (i, a) in [1, 10, 27].into_iter().enumerate() {
        println!("class {} (version {}..):", i + 1, a);
        for m1 in 0..3 {
            for m2 in m1..3 {
                for switch_len in 0..200 {
                    let l = "11111111111".to_string();
                    let m = ['1']
                        .into_iter()
                        .cycle()
                        .take(switch_len)
                        .collect::<String>();
                    let n = "1111111111111111".to_string();

                    let mode1 = modes[m1];
                    let mode2 = modes[m2];

                    let single = make_token_stream(vec![(mode1, format!("{}{}{}", l, m, n))]);
                    let multi = make_token_stream(vec![(mode1, l), (mode2, m), (mode1, n)]);

                    if bit_overhead(&single, a) > bit_overhead(&multi, a) {
                        let n = ["ascii", "alphanumeric", "numeric"];
                        println!(
                            "{}-{}-{} beats only {} at {} characters",
                            n[m1], n[m2], n[m1], n[m1], switch_len,
                        );
                        break;
                    }
                }
            }
        }
        for switch_len in 0..200 {
            let l = "11111111111".to_string();
            let m = ['1']
                .into_iter()
                .cycle()
                .take(switch_len)
                .collect::<String>();
            let n = "1111111111111111".to_string();

            let single =
                make_token_stream(vec![(ASCII, l.clone()), (AlphaNum, format!("{}{}", m, n))]);
            let multi = make_token_stream(vec![(ASCII, l), (Numeric, m), (AlphaNum, n)]);

            if bit_overhead(&single, a) > bit_overhead(&multi, a) {
                println!(
                    "ascii-num-aln beats an immediate switch to aln at {} characters",
                    switch_len,
                );
                break;
            }
        }
    }
}

// returns the smallest subset x is part of:
// Numeric ⊂ AlphaNum ⊂ ASCII
// to use for a "greedy" mode-switch algorithm
fn char_status(x: char) -> Option<Mode> {
    Some(if is_numeric(x) {
        // ascii, alphanumeric and numeric
        Numeric
    } else if is_alphanum(x) {
        // ascii and alphanumeric
        AlphaNum
    } else if x.is_ascii() {
        // only ascii
        ASCII
    } else {
        return None;
    })
}

#[inline]
fn is_alphanum(x: char) -> bool {
    find_alphanum(x).is_some()
}

#[inline]
fn is_numeric(x: char) -> bool {
    x.is_ascii_digit()
}

fn optimize_mode(input: String) -> Vec<(Mode, usize)> {
    // all right, so, how do i want to do this?
    // the plan is to mark all characters with their mode,
    // and then "look for patterns" in the data.
    // the issue is that version size determines the "economy",
    // so i end up with a cyclic definition:
    // version implies economy implies data size implies version.
    // maybe just calculate all three and decide afterwards which one is best?

    // mark characters in input
    let mut char_modes = input
        .chars()
        .map(|x| char_status(x).expect("optimize_mode: invalid character in input"));

    // save run lengths in a vector
    let mut mode_run_lengths: Vec<(Mode, usize)> = Vec::new();

    // mode of the first character
    let (mut run_mode, mut run_count): (Mode, usize) = (
        if let Some(mode) = char_modes.next() {
            mode
        } else {
            // the input was an empty string
            // we return an empty vector
            eprintln!("warning: input is empty!");
            return vec![];
        },
        1,
    );

    // fill in the remaining run lengths
    for mode in char_modes {
        if mode == run_mode {
            run_count += 1;
        } else {
            mode_run_lengths.push((run_mode, run_count));
            (run_mode, run_count) = (mode, 1);
        }
    }

    // mode optimization procedure
    for class in MODE_ECONOMY.into_iter() {
        // the original mode economy table (generated by
        // compute_bit_hypothetical) does not seem right to me,
        // but i can't spot the error. going with this for now.
        // note that class[1] and class[3] are ~equal...
        // i picked 0, 2, 3 instead of 0, 1, 2
        // because 3 is more pessimistic
        let [aln_to_asc, _, num_to_aln, num_to_asc] = class;

        // compare the current mode with the next one, to decide if
        // the current mode should actually be replaced with the next
        let mut mut_mode_runs = mode_run_lengths.clone();
        for (i, &(this_mode, this_run)) in mode_run_lengths.iter().enumerate() {
            // we're taking the next mode from the original vector instead of
            // its clone, to avoid borrow issues. this isn't a problem, since
            // we're only looking forward, at entries we couldn't have gone over yet
            let next_mode = if let Some(&(mode, _)) = mode_run_lengths.get(i + 1) {
                mode
            } else {
                // there are no more modes after this,
                // so no more analysis to be done
                break;
            };

            // is this mode ⊂ the next mode? it's time to do data analysis.
            let heuristic: usize = match (this_mode, next_mode) {
                (AlphaNum, ASCII) => aln_to_asc,
                (Numeric, ASCII) => num_to_asc,
                (Numeric, AlphaNum) => num_to_aln,
                _ => {
                    // our current mode is a superset of the next,
                    // and we can't convert it "downwards"

                    // double checking to make sure that the data isn't bad
                    assert!(
                        this_mode != next_mode,
                        "optimize_mode: consecutive modes of the same type"
                    );

                    continue;
                }
            };

            if this_run < heuristic {
                // too few characters to motivate switching to the current mode

                // what in the world do i do if i have a single number,
                // followed by a single alphanumeric character, and then ascii??
                // how would i handle that? am i better off using .windows()
                // than just looking ahead to the next mode? should i
                // actually be looking at run lengths to begin with???

                mut_mode_runs[i] = (next_mode, this_run);
            }
        }

        todo!("do something with mut_mode_runs here before it's dropped")
    }

    todo!("not finished")
}

// verified accurate
// returns the number of bits it takes to print `count` characters
// in a given mode and size class of qr code
fn bit_cost(count: usize, class: usize, mode: Mode) -> usize {
    4 + match mode {
        Numeric => 4 + [10, 12, 14][class] + ((10 * count + 1) as f32 / 3.0).round() as usize,
        AlphaNum => 4 + [9, 11, 13][class] + 11 * (count / 2) + 6 * (count % 2),
        ASCII => 4 + [8, 16, 16][class] + 8 * count,
        Kanji => todo!("refer to kanji bit information"),
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

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
                    0 => string_to_numeric(data),
                    1 => string_to_alphanum(data),
                    2 => string_to_ascii(data),
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

    #[test]
    fn test_bit_cost() {
        for count in 0..5000 {
            for mode in [Numeric, AlphaNum, ASCII].into_iter() {
                let string = vec![(mode, "1".chars().cycle().take(count).collect())];
                let stream = make_token_stream(string);
                let overhead = bit_overhead_template(&stream);
                for (class, version) in [1, 10, 27].into_iter().enumerate() {
                    assert!(
                        bit_cost(count, class, mode) == compute_bit_overhead(overhead, version),
                        "failure at {:?}, count {}, class {}: bit_cost = {}, actual overhead = {}",
                        mode,
                        count,
                        class,
                        bit_cost(count, class, mode),
                        compute_bit_overhead(overhead, version)
                    );
                }
            }
        }
    }
}
