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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
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

impl Mode {
    pub(crate) const LIST: [Self; 3] = [ASCII, AlphaNum, Numeric];
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
        .map(|x| find_alphanum(x).expect("invalid alphanumeric input!"))
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

pub(crate) fn compute_bit_hypothetical() {
    let modes = Mode::LIST;
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

#[allow(unused_variables, unreachable_code, unused_mut)]
pub(crate) fn optimize_mode(input: String) -> Vec<(Mode, String)> {
    {
        // skip over the work in progress so
        // the qr generator routine still works
        let skip = true;
        if skip {
            return vec![(ASCII, input)];
        }
    }

    // all right, so, how do i want to do this?
    // the plan is to mark all characters with their mode,
    // and then "look for patterns" in the data.
    // the issue is that version size determines the "economy",
    // so i end up with a cyclic definition:
    // version implies economy implies data size implies version.
    // maybe just calculate all three and decide afterwards which one is best?

    // mark characters in input based on whether they're part of the
    // alphanumeric set (left bit) and numeric set (right bit)
    let mut char_modes = input
        .chars()
        .map(|x| char_status(x).expect("optimize_mode: invalid character in input"));

    /*
    flipping between modes:

        num              _______     __
        aln        _____/       \   /  \____
        asc  _____/              \_/        \_____

    if switching to num isn't worth it, can i regard the "underlying"
    alphanumeric sequence ignoring numerals? i.e., will switching once
    always be more "worth it" than switching twice, no matter what?
    is that a transitive property?

    idea: check when switching to num is worth it, "flatten" the "peaks"
    that aren't (into alphanumeric), then analyze it again

    in this scenario:

        num          _______
        aln        _/
        asc  _____/

    the num section is intuitively "worth it" (by what metric?), so
    the aln character needs to be converted "down" to ascii

    actually unsure: under what circumstances are asc-asc-num solutions
    preferable to asc-aln-aln?? and at what point is asc-aln-num worth it?
    is there a way to just brute-force solutions instead of solving this?


    if a num section is neighbored by
    */

    // save run lengths in a vector
    let mut mode_run_lengths = Vec::new();

    // mode of the first character
    let mut run_mode = if let Some(mode) = char_modes.next() {
        mode
    } else {
        // the input was an empty string
        // we return an empty vector
        eprintln!("warning: input is empty!");
        return vec![];
    };
    let mut run_count = 1;

    // fill in the remaining run lengths
    for mode in char_modes {
        if mode == run_mode {
            run_count += 1;
        } else {
            mode_run_lengths.push((run_mode, run_count));
            (run_mode, run_count) = (mode, 1);
        }
    }
    mode_run_lengths.push((run_mode, run_count));

    // mode optimization procedure
    for class in MODE_ECONOMY.into_iter() {
        // the original mode economy table (generated by
        // compute_bit_hypothetical) does not seem right to me,
        // but i can't spot the error. going with this for now.
        // note that class[1] and class[3] are ~equal...
        // i picked 0, 2, 3 instead of 0, 1, 2
        // because 3 is more pessimistic
        let [aln_to_asc, _, num_to_aln, num_to_asc] = class;

        let mut mrl = mode_run_lengths.clone();

        // check two things:
        // do we change the following mode based on the current?
        // do we change the current mode based on the following?
        for i in 0..(mrl.len() - 1) {
            match mrl[i] {
                // no need to regard the length of the ascii block
                // because those chars only exist in one mode
                (ASCII, _) => match mrl[i + 1] {
                    (AlphaNum, len2) => {
                        if len2 < aln_to_asc {
                            // if the alphanumeric section is too short,
                            // look even further forward to see if there's
                            // a numeric section that can be optimized
                            if let Some((Numeric, len3)) = mrl.get(i + 2) {
                                todo!()
                            } else {
                                mrl[i + 1] = (ASCII, len2);
                            }
                        }
                    }
                    (Numeric, len2) => todo!(),
                    _ => panic!(),
                },
                (AlphaNum, len) => {}
                (Numeric, len) => {}
                _ => panic!(),
            }
        }

        /*
        // compare the current mode with the next one, to decide if
        // the current mode should actually be replaced with the next
        let mut mut_mode_runs = mode_run_lengths.clone();
        for (i, &(right_mode, right_run)) in mode_run_lengths.iter().enumerate().rev() {
            // we're taking the next mode from the original vector instead of
            // its clone, to avoid borrow issues. this isn't a problem, since
            // we're only looking forward, at entries we couldn't have gone over yet
            let (left_mode, left_run) = if let Some(&item) = mode_run_lengths.get(i - 1) {
                item
            } else {
                // there are no more modes after this,
                // so no more analysis to be done
                break;
            };

            // // is this mode ⊂ the next mode? it's time to do data analysis.
            // let heuristic: usize = match (right_mode, left_mode) {
            //     (AlphaNum, ASCII) => aln_to_asc,
            //     (Numeric, ASCII) => num_to_asc,
            //     (Numeric, AlphaNum) => num_to_aln,
            //     _ => {
            //         // our current mode is a superset of the next,
            //         // and we can't convert it "downwards"

            //         // double checking to make sure that the data isn't bad
            //         assert!(
            //             right_mode != left_mode,
            //             "optimize_mode: consecutive modes of the same type"
            //         );

            //         continue;
            //     }
            // };

            // if right_run < heuristic {
            //     // too few characters to motivate switching to the current mode

            //     // what in the world do i do if i have a single number,
            //     // followed by a single alphanumeric character, and then ascii??
            //     // how would i handle that? am i better off using .windows()
            //     // than just looking ahead to the next mode? should i
            //     // actually be looking at run lengths to begin with???

            //     mut_mode_runs[i] = (left_mode, right_run);
            // }
        }
        todo!("do something with mut_mode_runs here before it's dropped")
        */
    }

    todo!("not finished")
}

#[allow(unused_variables, unreachable_code, unused_mut)]
pub(crate) fn wip_alt_optimize_mode(input: String) -> Vec<(Mode, String)> {
    // is it alphanumeric? / is it numeric?
    let stat_vec: Vec<(bool, bool)> = input
        .chars()
        .map(
            |x| match char_status(x).expect("invalid character in input") {
                ASCII => (false, false),
                AlphaNum => (true, false),
                Numeric => (true, true),
                _ => panic!(),
            },
        )
        .collect::<Vec<_>>();

    // returns the length of run of a type of character
    // if aln is true, alphanumeric (> numeric)
    // if false, numeric
    let run = |index: usize, aln: bool| {
        let mut output = 0;
        'count: {
            for i in index..stat_vec.len() {
                let (is_aln, is_num) = stat_vec[i];
                if (aln && is_aln) || (!aln && is_num) {
                    // we're on the kind of type of character we're looking for
                    output += 1;
                } else {
                    // the run of characters has ended
                    break 'count;
                }
            }
            // if the end of the string is reached,
            // output is too low; we correct it
            output += 1;
        }
        output
    };

    // // alphanumeric, numeric:
    // // start index of block, length of block
    // // alphanumeric > numeric so any numeric block will also be
    // // part of an alphanumeric block
    let mut run_lengths: (Vec<(usize, usize)>, Vec<(usize, usize)>) = (vec![], vec![]);

    let status = stat_vec.iter().enumerate();
    for (index, &(aln, num)) in status {
        'nums: {
            if num {
                if let Some((start, len)) = run_lengths.0.last() {
                    if start + len > index {
                        // we're inside of an already-registered run
                        break 'nums;
                    }
                }
                let num_run = run(index, false);
                run_lengths.0.push((index, num_run));
            }
        }
        'alns: {
            if aln {
                if let Some((start, len)) = run_lengths.1.last() {
                    if start + len > index {
                        break 'alns;
                    }
                }
                let aln_run = run(index, true);
                run_lengths.1.push((index, aln_run));
            }
        }
    }

    // panic!(
    //     "{:?}\n{:?}\n{:?}",
    //     input.chars().collect::<Vec<_>>(),
    //     run_lengths.0,
    //     run_lengths.1
    // );

    // mode optimization procedure
    for class in MODE_ECONOMY.into_iter() {
        let [aln_to_asc, _, num_to_aln, num_to_asc] = class;
    }

    todo!("not finished")
}

// weights for graph traversal
// (index of next element, weights to reach it)
// index is necessary because the output list is abbreviated
// required to use floats because this approximation
// is the only way to form a (weighted) graph -
// otherwise distances would vary depending on
// where you started from

// bool: are they the same?
// entries: -> num, -> aln, -> asc
type ModeSwitchWeights = (bool, [Option<f32>; 3]);

#[allow(unused_variables, unreachable_code, unused_mut)]
fn make_mode_graph(input: Vec<Mode>) -> Vec<(usize, ModeSwitchWeights)> {
    let mut output: Vec<(usize, ModeSwitchWeights)> = vec![];

    let mut mode_iter = input.into_iter().enumerate();
    let (index, mut current_mode) = mode_iter.next().expect("mode vector is empty!");

    let mut counts = <[f32; 3]>::default();
    for (new_index, new_mode) in mode_iter {
        output.push((new_index, mode_to_mode_weights(current_mode, new_mode, 0)));
    }

    output
}

fn vec_add_assign<T, U, const N: usize>(lhs: &mut [T; N], rhs: [U; N])
where
    T: for<'a> std::ops::AddAssign<&'a U>,
{
    for i in 0..N {
        lhs[i] += &rhs[i];
    }
}

fn mode_to_mode_weights(start: Mode, end: Mode, class: usize) -> ModeSwitchWeights {
    let from = u32::from(start == end);
    (
        start == end,
        [
            // to num
            (end == Numeric).then_some(((4 + [10, 12, 14][class]) * from) as f32 + 3.3),
            // to aln
            (end <= AlphaNum).then_some(((4 + [9, 11, 13][class]) * from) as f32 + 5.5),
            // to ascii
            Some(((4 + [8, 16, 16][class] + 8) * from) as f32),
        ],
    )
}

mod a_star {
    #![allow(unused_variables, unreachable_code, unused_mut)]

    use super::Mode::{self, *};
    use std::collections::{HashMap, HashSet};

    // char index, char type (i.e. subset)
    // NOTE: num chars have 3 nodes, aln chars have 2, asc 1
    type NodeId = (usize, Mode);

    type Path = Vec<NodeId>;
    type Map<T> = HashMap<NodeId, T>;
    type Cost = f32;

    //     function reconstruct_path(cameFrom, current)
    fn reconstruct_path(came_from: Map<NodeId>, current: NodeId) -> Path {
        //     total_path := {current}
        let mut total_path: Path = vec![current];

        //     while current in cameFrom.Keys:
        let mut current = current;
        while let Some(&x) = came_from.get(&current) {
            //         current := cameFrom[current]
            current = x;
            //         total_path.prepend(current)
            total_path.push(current);
        }

        //     return total_path
        total_path.reverse();
        total_path
    }

    fn get(score_map: &mut Map<Cost>, node: NodeId) -> Cost {
        // as per the IEEE 754 standard, adding a finite number to infinity
        // makes infinity. meaning, this is well-behaved
        score_map.entry(node).or_insert(Cost::INFINITY).clone()
    }

    fn entry_mut<'a, T>(node_map: &'a mut Map<T>, node: &NodeId) -> &'a mut T {
        node_map.get_mut(node).expect("hashmap is empty!")
    }

    // the heuristic function
    // estimated cost of reaching the finish from here
    fn h(node: NodeId) -> Cost {
        // plan: the calculation is "go along the current mode
        // as far as possible, then switch to ascii for the remainder"
        todo!()
    }

    // distance between neighbors
    // todo: figure out how to integrate size classes
    fn d(from: NodeId, to: NodeId) -> Cost {
        if from.0 + 1 == to.0 {
            (if from.1 != to.1 {
                // add length of char count indicator
                crate::qr_standard::cc_indicator_bit_size(
                    {
                        // help!! what class do i pick?
                        2
                    },
                    to.1,
                )
            } else {
                0
            }) as Cost
                + match to.1 {
                    ASCII => 8.0,
                    AlphaNum => 5.5,
                    Numeric => 3.3,
                    Kanji => todo!(),
                }
        } else {
            Cost::INFINITY
        }
    }

    // // A* finds a path from start to goal.
    // // h is the heuristic function. h(n) estimates the cost to reach goal from node n.
    // function A_Star(start, goal, h)
    fn a_star<F>(start: NodeId, goal: NodeId, h: F) -> Option<Path>
    where
        F: Fn(NodeId) -> Cost,
    {
        //     // The set of discovered nodes that may need to be (re-)expanded.
        //     // Initially, only the start node is known.
        //     // This is usually implemented as a min-heap or priority queue rather than a hash-set.
        //     openSet := {start}
        let mut open_set: HashSet<NodeId> = HashSet::from([start]);

        //     // For node n, cameFrom[n] is the node immediately preceding it on the cheapest path from the start
        //     // to n currently known.
        //     cameFrom := an empty map
        let mut came_from: Map<NodeId> = HashMap::new();

        //     // For node n, gScore[n] is the cost of the cheapest path from start to n currently known.
        //     gScore := map with default value of Infinity
        //     gScore[start] := 0
        let mut g_score: Map<Cost> = HashMap::new();

        //     // For node n, fScore[n] := gScore[n] + h(n). fScore[n] represents our current best guess as to
        //     // how cheap a path could be from start to finish if it goes through n.
        //     fScore := map with default value of Infinity
        //     fScore[start] := h(start)
        let mut f_score: Map<Cost> = HashMap::from([(start, h(start))]);

        //     while openSet is not empty
        while !open_set.is_empty() {
            //         // This operation can occur in O(Log(N)) time if openSet is a min-heap or a priority queue
            //         current := the node in openSet having the lowest fScore[] value
            let mut current = {
                let mut nodes = open_set.iter();
                let mut best_node = *nodes.next().expect("set is empty!");
                let mut best_score = get(&mut f_score, best_node);
                for &candidate_node in nodes {
                    let try_score = get(&mut f_score, candidate_node);
                    if try_score < best_score {
                        best_node = candidate_node;
                        best_score = try_score;
                    }
                }
                best_node
            };
            //         if current = goal
            if current == goal {
                //             return reconstruct_path(cameFrom, current)
                return Some(reconstruct_path(came_from, current));
            }

            //         openSet.Remove(current)
            open_set.remove(&current);

            //         for each neighbor of current
            for neighbor in [todo!("find neighbors")] {
                //             // d(current,neighbor) is the weight of the edge from current to neighbor
                //             // tentative_gScore is the distance from start to the neighbor through current
                //             tentative_gScore := gScore[current] + d(current, neighbor)
                let tentative_g_score = get(&mut g_score, current) + todo!("implement d") as Cost;

                //             if tentative_gScore < gScore[neighbor]
                if tentative_g_score < get(&mut g_score, neighbor) {
                    //                 // This path to neighbor is better than any previous one. Record it!
                    //                 cameFrom[neighbor] := current
                    *entry_mut(&mut came_from, &neighbor) = current;
                    //                 gScore[neighbor] := tentative_gScore
                    *entry_mut(&mut g_score, &neighbor) = tentative_g_score;
                    //                 fScore[neighbor] := tentative_gScore + h(neighbor)
                    *entry_mut(&mut f_score, &neighbor) = tentative_g_score + h(neighbor);

                    //                 if neighbor not in openSet
                    //                     openSet.add(neighbor)
                    open_set.insert(
                        // note that insert() does this if clause already,
                        // so no need to implement it
                        neighbor,
                    );
                }
            }
        }

        //     // Open set is empty but goal was never reached
        //     return failure
        None
    }
}

mod good_star {
    #![allow(unused_variables, unused_assignments, unreachable_code, unused_mut)]

    use super::Mode::{self, *};

    // a simpler search algorithm than a*
    // compute the "g score" of every node,
    // backtrack and pick the lowest value

    // if two characters are of the same type,
    // there is no reason to switch modes

    // any message of length n has at most
    // 6n-6 edges (alternating between aln-asc)

    // the cheapest known way to reach a character
    // we approximate the cost of a single aln/num
    // char as 11/2 and 10/3 respectively, but if we
    // multiply it by 6 it makes an integer
    // 10/3 -> 20
    // 11/2 -> 33
    // 8 -> 48

    type Cost = u32;
    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    struct TaggedNode(Cost, Option<Mode>);

    impl Default for TaggedNode {
        fn default() -> Self {
            TaggedNode(u32::MAX, None)
        }
    }

    impl PartialOrd for TaggedNode {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.0.partial_cmp(&other.0)
        }
    }

    impl Ord for TaggedNode {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.0.cmp(&other.0)
        }
    }

    // the 1, 2 or 3 nodes associated with
    // a character
    #[derive(Clone, Copy)]
    struct CharNodes(Mode, [TaggedNode; 3]);

    impl CharNodes {
        fn get(&self, category: Mode) -> Option<TaggedNode> {
            let index = category.index();
            if index > self.max() {
                None
            } else {
                Some(self.1[index])
            }
        }

        fn set_min(&mut self, category: Mode, value: TaggedNode) {
            let index = category.index();
            if index > self.max() || self.1[index] <= value {
                return;
            } else {
                self.1[index] = value;
            }
        }

        // max index for a character's node list
        fn max(&self) -> usize {
            self.0.index()
        }

        // propagate best score + edge weight
        fn score_from_predecessor(&mut self, from: &Self, class: u8) {
            let modes = Mode::LIST;
            // check each node we're going from
            for &from_mode in modes.iter() {
                // if that node exists,
                if let Some(TaggedNode(from_score, _)) = from.get(from_mode) {
                    // check each node we're moving towards
                    for &to_mode in modes.iter() {
                        // and if THAT node exists,
                        if to_mode.index() <= self.0.index() {
                            // are we moving between two nodes of the same type?
                            let same_subset = from_mode == to_mode;

                            // calculate the value of the node we're moving towards
                            let to_score = from_score + edge_weight(to_mode, same_subset, class);

                            // if the score is lower than what's already there,
                            // i.e. we're on a more optimal path, replace it
                            if to_score < self.1[to_mode.index()].0 {
                                self.1[to_mode.index()] = TaggedNode(to_score, Some(from_mode));
                            }
                        } else {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        }
    }

    // the nodes corresponding to the full message
    type Graph = Vec<CharNodes>;

    // averaged distance between neighbors
    fn edge_weight(to_mode: Mode, same_subset: bool, class: u8) -> Cost {
        (if !same_subset {
            // we multiply by 6 to get rid of decimals
            // the 4 is the size of the mode indicator
            6 * (4 + crate::qr_standard::cc_indicator_bit_size(class, to_mode)) as Cost
        } else {
            0
        }) + match to_mode {
            // 6 * 8
            ASCII => 48,
            // 6 * 11/2
            AlphaNum => 33,
            // 6 * 10/3
            Numeric => 20,
            Kanji => todo!(),
        }
    }

    // creates a graph of nodes along with their "g scores"
    fn create_graph(mode_vec: &Vec<Mode>, class: u8) -> Graph {
        let mut mode_iter = mode_vec.iter();
        let mut current_scores = [TaggedNode::default(); 3];

        // first character is a special case - the "same subset" parameter
        // is false for all modes
        let init_mode = *mode_iter.next().expect("mode vector is empty");
        for (init_score, to_mode) in current_scores
            .iter_mut()
            .zip(Mode::LIST)
            .take(init_mode.index() + 1)
        {
            *init_score = TaggedNode(edge_weight(to_mode, false, class), None);
        }

        let mut current_nodes = CharNodes(init_mode, current_scores);
        let mut output: Graph = vec![current_nodes];

        let mut previous_nodes: CharNodes;
        for &mode in mode_iter {
            previous_nodes = current_nodes;
            current_nodes = CharNodes(mode, [TaggedNode::default(); 3]);
            current_nodes.score_from_predecessor(&previous_nodes, class);
            output.push(current_nodes);
        }

        output
    }

    fn optimal_path(graph: &Graph) -> Vec<Mode> {
        let mut graph_backwards = graph.iter();

        for (i, &x) in graph_backwards.clone().enumerate() {
            println!("old graph index {} - {:?}", i, x.0);
        }

        let mut output = std::collections::VecDeque::new();

        let mut current_best_mode = {
            let character = *graph_backwards.next_back().expect("uhh");
            let mut best_mode = ASCII;
            let mut best_score = character.get(ASCII).unwrap().0;
            for mode in [AlphaNum, Numeric] {
                if let Some(TaggedNode(score, _)) = character.get(mode) {
                    if score < best_score {
                        best_mode = mode;
                        best_score = score;
                    }
                } else {
                    break;
                }
            }
            best_mode
        };

        output.push_front(current_best_mode);
        // let mut output = std::collections::VecDeque::from([current_best_mode]);

        // println!("\n");
        // for (i, &x) in graph_backwards.clone().enumerate() {
        //     println!("new graph index {} - {:?}", i, x.0);
        // }

        // println!("🤔 {:?}", output);

        // for (i, &character) in graph_backwards.enumerate().rev() {
        //     if let Some(tagged_node) = get(character, current_best_mode) {
        //         if let Some(mode) = tagged_node.1 {
        //             current_best_mode = mode;
        //             output.push_front(current_best_mode);
        //         } else {
        //             // output.push_front(best_mode);
        //             break;
        //         }
        //     } else {
        //         eprintln!("node does not exist {}", i);
        //     }
        //     if i > graph.len() - 10 {
        //         println!("💥 {} {:?}", i, output);
        //     }
        // }

        while let Some(&character) = graph_backwards.next_back() {
            if let Some(tagged_node) = character.get(current_best_mode) {
                if let Some(mode) = tagged_node.1 {
                    current_best_mode = mode;
                    output.push_front(current_best_mode);
                } else {
                    // output.push_front(current_best_mode);
                    break;
                }
            }
        }

        let mout = Vec::from(output.clone());

        for (i, &content) in mout.iter().enumerate() {
            println!(
                "{:3} → {:?} ⊇ {:?} ({:?})",
                i,
                content,
                graph[i].0,
                content >= graph[i].0
            );
            // assert!(content > graph[i].0, "discrepancy at index {}", i);
        }

        // assert!(
        //     output.len() == graph.len(),
        //     "path len is {} but graph len is {}",
        //     output.len(),
        //     graph.len()
        // );

        mout
    }

    impl Mode {
        fn index(&self) -> usize {
            let mode = *self;
            match mode {
                ASCII => 0,
                AlphaNum => 1,
                Numeric => 2,
                Kanji => todo!(),
            }
        }
    }

    mod tests {

        use super::*;

        #[test]
        fn create_graph_nocapture() {
            let scramble = |x: usize| {
                (
                    // ((x + 1) * x)
                    if {
                        [1, 24].contains(&x)
                        // x.count_ones().count_zeros() % 2 == 0
                    } {
                        0
                        // x * 2 + 1
                    } else if {
                        //

                        [17, 23].contains(&x)

                        //
                    } {
                        // (x / 8).pow(3)
                        1
                    } else {
                        2
                    }
                    // if x == 8 { 2 } else { 0 }
                    // if x.count_ones() % 2 == 0 {
                    //     (x % 5) + (x % 7)
                    // } else {
                    //     (x * x + 1) << (x / 2)
                    // }
                ) % 3
            };

            let mode_vec = (0..10).map(|x| Mode::LIST[scramble(x)]).collect::<Vec<_>>();

            println!("modes:\n{:?}", mode_vec);
            for class in 0..3 {
                println!("class {}", class);
                for mode in Mode::LIST {
                    println!(
                        "-> {:?}: {}",
                        mode,
                        crate::qr_standard::cc_indicator_bit_size(class, mode)
                    );
                }

                let a = {
                    if true {
                        create_graph(&mode_vec, class)
                    } else {
                        let mut a = vec![];
                        for i in 0..5 {
                            a.push(CharNodes(
                                Mode::LIST[(i / 3) % 2 + 1],
                                [TaggedNode(i as u32, Some(Mode::LIST[(i / 3) % 2])); 3],
                            ))
                        }
                        a
                    }
                };

                helpy_print_graph(&a);
                println!("\n");
                println!("{:?}", optimal_path(&a));
            }
        }

        fn helpy_print_graph(graph: &Graph) {
            for &CharNodes(mode, data) in graph {
                println!("{:10?} ---", mode);
                for (i, &k) in data.iter().enumerate() {
                    if k < TaggedNode::default() {
                        let to_mode = Mode::LIST[i];
                        let mode_name = if let Some(mode) = k.1 {
                            format!("{:?}", mode)
                        } else {
                            "none :(".to_string()
                        };
                        println!(
                            "{:?} -> {:4} (from {})",
                            to_mode,
                            ((k.0 as f32) / 6.0).round() as i32,
                            mode_name,
                        );
                    }
                }
                println!("---");
            }
        }
    }
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

// not sure what to call this function...
// runs through every possible solution
// to the scenario below
pub(crate) fn mode_switch_brute_force_analysis() {
    // how many chars to test for
    // let limit = 50;

    /*
    testing this scenario:

    num              _______
    aln        _____/       \_____
    asc  _____/                   \______

    the possible solutions are:
    0. asc - aln - num - aln - asc     max switching
    1. asc - aln      ...    - asc     lower num to aln
    2. asc ...   - num - aln - asc     lower first aln to asc
    3. asc - aln - num - asc ...       lower last aln to asc
    4. asc ...   - num - asc ...       lower both aln to asc
    5. asc                   ...       all asc
    */

    for limit in [10, 20, 50] {
        println!("limit {}:", limit);
        for class in 0..3 {
            let bit = |count, mode| bit_cost(count, class, mode);
            let mut scores = [0usize; 6];

            for first_aln in 0..limit {
                for num in 0..limit {
                    for last_aln in 0..limit {
                        let solutions = [
                            // 0: max switching
                            bit(10, ASCII)
                                + bit(first_aln, AlphaNum)
                                + bit(num, Numeric)
                                + bit(last_aln, AlphaNum)
                                + bit(10, ASCII),
                            // 1: lower num to aln
                            bit(10, ASCII)
                                + bit(first_aln + num + last_aln, AlphaNum)
                                + bit(10, ASCII),
                            // 2: lower first aln to asc
                            bit(10 + first_aln, ASCII)
                                + bit(num, Numeric)
                                + bit(last_aln, AlphaNum)
                                + bit(10, ASCII),
                            // 3: lower last aln to asc
                            bit(10, ASCII)
                                + bit(first_aln, AlphaNum)
                                + bit(num, Numeric)
                                + bit(last_aln + 10, ASCII),
                            // 4: lower both aln to asc
                            bit(10 + first_aln, ASCII)
                                + bit(num, Numeric)
                                + bit(last_aln + 10, ASCII),
                            // 5: all asc
                            bit(10 + first_aln + num + last_aln + 10, ASCII),
                        ];

                        let mut best_index = 0;
                        let mut lowest_cost = usize::MAX;

                        for (i, cost) in solutions.into_iter().enumerate() {
                            if cost < lowest_cost {
                                lowest_cost = cost;
                                best_index = i;
                            }
                        }

                        scores[best_index] += 1;
                    }
                }
            }

            let scores: Vec<f32> = scores
                .into_iter()
                .map(|x| x as f32 / (limit.pow(3) as f32))
                .collect();
            println!("class {}: {:?}", class, scores);
        }
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
