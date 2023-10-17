use core::num;

// the centers of alignment patterns in both x and y dimensions
pub(super) const AP_COORD_INDICES: [&[usize]; 40] = [
    &[],
    &[6, 18],
    &[6, 22],
    &[6, 26],
    &[6, 30],
    &[6, 34],
    &[6, 22, 38],
    &[6, 24, 42],
    &[6, 26, 46],
    &[6, 28, 50],
    &[6, 30, 54],
    &[6, 32, 58],
    &[6, 34, 62],
    &[6, 26, 46, 66],
    &[6, 26, 48, 70],
    &[6, 26, 50, 74],
    &[6, 30, 54, 78],
    &[6, 30, 56, 82],
    &[6, 30, 58, 86],
    &[6, 34, 62, 90],
    &[6, 28, 50, 72, 94],
    &[6, 26, 50, 74, 98],
    &[6, 30, 54, 78, 102],
    &[6, 28, 54, 80, 106],
    &[6, 32, 58, 84, 110],
    &[6, 30, 58, 86, 114],
    &[6, 34, 62, 90, 118],
    &[6, 26, 50, 74, 98, 122],
    &[6, 30, 54, 78, 102, 126],
    &[6, 26, 52, 78, 104, 130],
    &[6, 30, 56, 82, 108, 134],
    &[6, 34, 60, 86, 112, 138],
    &[6, 30, 58, 86, 114, 142],
    &[6, 34, 62, 90, 118, 146],
    &[6, 30, 54, 78, 102, 126, 150],
    &[6, 24, 50, 76, 102, 128, 154],
    &[6, 28, 54, 80, 106, 132, 158],
    &[6, 32, 58, 84, 110, 136, 162],
    &[6, 26, 54, 82, 110, 138, 166],
    &[6, 30, 58, 86, 114, 142, 170],
];

pub fn alignment_pattern_coords(version: u32) -> Vec<(usize, usize)> {
    if !(1..=40).contains(&version) {
        panic!()
    }
    let indices = AP_COORD_INDICES[version as usize - 1];
    let mut output = Vec::new();

    for &x in indices {
        for &y in indices {
            if [x, y].contains(&6) {
                if x == y || [x, y].contains(&indices.last().unwrap()) {
                    continue;
                }
            }
            output.push((x, y));
        }
    }
    output
}

// number of codewords in a given code version
pub const CODEWORDS: [u32; 40] = [
    26, 44, 70, 100, 134, 172, 196, 242, 292, 346, 404, 466, 532, 581, 655, 733, 815, 901, 991,
    1085, 1156, 1258, 1364, 1474, 1588, 1706, 1828, 1921, 2051, 2185, 2323, 2465, 2611, 2761, 2876,
    3034, 3196, 3362, 3532, 3706,
];

// the possible amounts of error correction codewords, ordered by size
pub const ERROR_CORRECTION_CODEWORDS: [u32; 31] = [
    7, 10, 13, 15, 16, 17, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 40, 42, 44, 46, 48, 50, 52, 54,
    56, 58, 60, 62, 64, 66, 68,
];

// reverse-lookup to find the index for precomputed.rs > RDSM_GENERATOR_POLYNOMIALS
pub fn find_errc(input: usize) -> Option<usize> {
    Some(
        (input
            - match input {
                7 | 17 => 7,
                13 | 15 => 9,
                x if x % 2 == 1 => return None,
                18..=36 => 6,
                10 | 16 | 40..=68 => 8,
                _ => return None,
            })
            / 2,
    )
}

pub fn find_errc2(input: u32) -> Option<usize> {
    ERROR_CORRECTION_CODEWORDS.iter().position(|&a| a == input)
}

#[test]
fn test_errc() {
    for i in 0..70 {
        assert!(find_errc(i) == find_errc2(i as u32));
    }
}

// table of characters for the alphanumeric encoding, ordered by index
// the ascii indices are +48 for numbers, +55 for letters,
// and for special chars, -4, -1, -1, 3, 3, 4, 4, 4, 14
// (special chars have indices 36..=44 in this table)
pub(super) const ALPHANUMERIC_TABLE: [char; 45] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', ' ', '$',
    '%', '*', '+', '-', '.', '/', ':',
];

// tested, works
pub(super) fn find_alphanum(input: char) -> u16 {
    u16::from(match input {
        '0'..='9' => (input as u8) - 48,
        'A'..='Z' => (input as u8) - 55,
        ' ' => 36,
        '$' => 37,
        '%' => 38,
        '*' => 39,
        '+' => 40,
        '-' => 41,
        '.' => 42,
        '/' => 43,
        ':' => 44,
        _ => panic!("not alphanumeric"),
    })
}

pub fn find_alphanum2(input: char) -> Option<usize> {
    ALPHANUMERIC_TABLE.iter().position(|&a| a == input)
}

// remainder bits per version (pg. 21):
// 2..=6       7 bits
// 14..=20     3 bits
// 21..=27     4 bits
// 28..=34     3 bits
// all other versions 0 bits
pub fn remainder(version: u32) -> u8 {
    match version {
        2..=6 => 7,
        14..=20 | 28..=34 => 3,
        21..=27 => 4,
        _ => 0,
    }
}

// error correction data (pg. 41...)
// access with ERROR_CORRECTION_TABLE[version-1][correction level]
// correction levels are ordered L - M - Q - H
// data format is:
// ec block count, total codewords per block, data codewords per block
// if there is just one block variant, the other pair member will be None
pub(super) const EC_BLOCK_TABLE: &[[(usize, usize, usize, Option<(usize, usize, usize)>); 4]] = &[
    [
        // version 1
        (1, 26, 19, None),
        (1, 26, 16, None),
        (1, 26, 13, None),
        (1, 26, 9, None),
    ],
    [
        // version 2
        (1, 44, 34, None),
        (1, 44, 28, None),
        (1, 44, 22, None),
        (1, 44, 16, None),
    ],
    [
        // version 3
        (1, 70, 55, None),
        (1, 70, 44, None),
        (2, 35, 17, None),
        (2, 35, 13, None),
    ],
    [
        // version 4
        (1, 100, 80, None),
        (2, 50, 32, None),
        (2, 50, 24, None),
        (4, 25, 9, None),
    ],
    [
        // version 5
        (1, 134, 108, None),
        (2, 67, 43, None),
        (2, 33, 15, Some((2, 34, 16))),
        (2, 33, 11, Some((2, 34, 12))),
    ],
    [
        // version 6
        (2, 86, 68, None),
        (4, 43, 27, None),
        (4, 43, 19, None),
        (4, 43, 15, None),
    ],
];

pub const EC_BLOCK_STR: &[&str] = &[
    //
    "version 7
2 98 78
4 49 31
2 32 14, 4 33 15
4 39 13, 1 40 14",
    //
    "version 8
2 121 97
2 60 38, 2 61 39
4 40 18, 2 41 19
4 40 14, 2 41 15",
    //
    "version 9
2 146 116
3 58 36, 2 59 37
4 36 16, 4 37 17
4 36 12, 4 37 13",
    //
    "version 10
2 86 68, 2 87 69
4 69 43, 1 70 44
6 43 19, 2 44 20
6 43 15, 2 44 16",
    //
    "version 11
4 101 81
1 80 50, 4 81 51
4 50 22, 4 51 23
3 36 12, 8 37 13",
    //
    "version 12
2 116 92, 2 117 93
6 58 36, 2 59 37
4 46 20, 6 47 21
7 42 14, 4 43 15",
    //
    "version 13
4 133 107
8 59 37, 1 60 38
8 44 20, 4 45 21
12 33 11, 4 34 12",
    //
    "version 14
3 145 115, 1 146 116
4 64 40, 5 65 41
11 36 16, 5 37 17
11 36 12, 5 37 13",
    //
    "version 15
5 109 87, 1 110 88
5 65 41, 5 66 42
5 54 24, 7 55 25
11 36 12, 7 37 13",
    //
    "version 16
5 122 98, 1 123 99
7 73 45, 3 74 46
15 43 19, 2 44 20
3 45 15, 13 46 16",
    //
    "version 17
1 135 107, 5 136 108
10 74 46, 1 75 47
1 50 22, 15 51 23
2 42 14, 17 43 15",
    //
    "version 18
5 150 120, 1 151 121
9 69 43, 4 70 44
17 50 22, 1 51 23
2 42 14, 19 14 15",
    //
    "version 19
3 141 113, 4 142 114
3 70 44, 11 71 45
17 47 21, 4 48 22
9 39 13, 16 40 14",
    //
    "version 20
3 135 107, 5 136 108
3 67 41, 13 68 42
15 54 24, 5 55 25
15 43 15, 10 44 16",
    //
    "version 21
4 144 116, 4 145 117
17 68 42
17 50 22, 6 51 23
19 46 16, 6 47 17",
    //
    "version 22
2 139 111, 7 140 112
17 74 46
7 54 24, 16 55 25
34 37 13",
    //
    "version 23
4 151 121, 5 152 122
4 75 47, 14 76 48
11 54 24, 14 55 25
16 45 15, 14 46 16",
    //
    "version 24
6 147 117, 4 148 118
6 73 45, 14 74 46
11 54 24, 16 55 25
30 46 16, 2 47 17",
    //
    "version 25
8 132 106, 4 133 107
8 75 47, 13 76 48
7 54 15, 22 55 25
22 45 15, 13 46 16",
    //
    "version 26
10 142 114, 2 143 115
19 74 46, 4 75 47
28 50 22, 6 51 23
33 46 16, 4 47 17",
    //
    "version 27
8 152 122, 4 153 123
22 73 45, 3 74 46
8 53 23, 26 54 24
12 45 15, 28 46 16",
    //
    "version 28
3 147 117, 10 148 118
3 73 45, 23 74 46
4 54 24, 31 55 25
11 45 15, 31 46 16",
    //
    "version 29
7 146 116, 7 147 117
21 73 45, 7 74 46
1 53 23, 37 54 24
19 45 15, 26 46 16",
    //
    "version 30
5 145 115, 10 146 116
19 75 47, 10 76 48
15 54 24, 25 55 25
23 45 15, 25 46 16",
    //
    "version 31
13 145 115, 3 146 116
2 74 46, 29 75 47
42 54 24, 1 55 25
23 45 15, 28 46 16",
    //
    "version 32
17 145 115
10 74 46, 23 75 47
10 54 24, 35 55 25
19 45 15, 35 46 16",
    //
    "version 33
17 145 115, 1 146 116
14 74 46, 21 75 47
29 54 24, 19 55 25
11 45 15, 46 46 16",
    //
    "version 34
 13 145 115, 6 146 116
 14 74 46, 23 75 47
 44 54 24, 7 55 25
 59 46 16, 1 47 17",
    //
    "version 35
12 151 121, 7 152 122
12 75 47, 26 76 48
39 54 24, 14 55 25
22 45 15, 41 46 15",
    //
    "version 36
6 151 121, 14 152 122
6 75 47, 34 76 48
46 54 24, 10 55 25
2 45 15, 64 46 16",
    //
    "version 37
17 152 122, 4 153 123
29 74 46, 14 75 47
49 54 24, 10 55 25
24 45 15, 46 46 16",
    //
    "version 38
4 152 122, 18 153 123
13 74 46, 32 75 47
48 54 24, 14 55 25
42 45 15, 32 46 16",
    //
    "version 39
20 147 117, 4 148 118
40 75 47, 7 76 48
43 54 24, 22 55 25
10 45 15, 67 46 16",
    //
    "version 40
19 148 118, 6 149 119
18 75 47, 31 76 48
34 54 24, 34 55 25
20 45 15, 61 46 16",
    //
];

pub fn print_block_table() -> Option<()> {
    for version in &EC_BLOCK_STR[..10] {
        let mut lines = version.lines();
        let vnum = lines.next()?.split_once(' ')?.1;

        print!("[// version {}\n", vnum);

        for number_line in lines {
            if let Some((primary, secondary)) = number_line.split_once(',') {
                print!(
                    "({},Some(({}))),",
                    primary.trim().replace(' ', ","),
                    secondary.trim().replace(' ', ",")
                );
            } else {
                print!("({},None),", number_line.trim().replace(' ', ","));
            }
        }
        print!("],");
    }

    println!();
    Some(())
}
