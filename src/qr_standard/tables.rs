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
pub fn remainder_bits(version: u32) -> u8 {
    match version {
        2..=6 => 7,
        14..=20 | 28..=34 => 3,
        21..=27 => 4,
        _ => 0,
    }
}

pub type VersionBlockInfo = (usize, usize, usize, Option<(usize, usize, usize)>);

// error correction data (pg. 41...)
// access with ERROR_CORRECTION_TABLE[version-1][correction level]
// correction levels are ordered L - M - Q - H
// data format is:
// ec block count, total codewords per block, data codewords per block
// if there is just one block variant, the other pair member will be None
const EC_BLOCK_TABLE: [[VersionBlockInfo; 4]; 40] = [
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
    [
        // version 7
        (2, 98, 78, None),
        (4, 49, 31, None),
        (2, 32, 14, Some((4, 33, 15))),
        (4, 39, 13, Some((1, 40, 14))),
    ],
    [
        // version 8
        (2, 121, 97, None),
        (2, 60, 38, Some((2, 61, 39))),
        (4, 40, 18, Some((2, 41, 19))),
        (4, 40, 14, Some((2, 41, 15))),
    ],
    [
        // version 9
        (2, 146, 116, None),
        (3, 58, 36, Some((2, 59, 37))),
        (4, 36, 16, Some((4, 37, 17))),
        (4, 36, 12, Some((4, 37, 13))),
    ],
    [
        // version 10
        (2, 86, 68, Some((2, 87, 69))),
        (4, 69, 43, Some((1, 70, 44))),
        (6, 43, 19, Some((2, 44, 20))),
        (6, 43, 15, Some((2, 44, 16))),
    ],
    [
        // version 11
        (4, 101, 81, None),
        (1, 80, 50, Some((4, 81, 51))),
        (4, 50, 22, Some((4, 51, 23))),
        (3, 36, 12, Some((8, 37, 13))),
    ],
    [
        // version 12
        (2, 116, 92, Some((2, 117, 93))),
        (6, 58, 36, Some((2, 59, 37))),
        (4, 46, 20, Some((6, 47, 21))),
        (7, 42, 14, Some((4, 43, 15))),
    ],
    [
        // version 13
        (4, 133, 107, None),
        (8, 59, 37, Some((1, 60, 38))),
        (8, 44, 20, Some((4, 45, 21))),
        (12, 33, 11, Some((4, 34, 12))),
    ],
    [
        // version 14
        (3, 145, 115, Some((1, 146, 116))),
        (4, 64, 40, Some((5, 65, 41))),
        (11, 36, 16, Some((5, 37, 17))),
        (11, 36, 12, Some((5, 37, 13))),
    ],
    [
        // version 15
        (5, 109, 87, Some((1, 110, 88))),
        (5, 65, 41, Some((5, 66, 42))),
        (5, 54, 24, Some((7, 55, 25))),
        (11, 36, 12, Some((7, 37, 13))),
    ],
    [
        // version 16
        (5, 122, 98, Some((1, 123, 99))),
        (7, 73, 45, Some((3, 74, 46))),
        (15, 43, 19, Some((2, 44, 20))),
        (3, 45, 15, Some((13, 46, 16))),
    ],
    [
        // version 17
        (1, 135, 107, Some((5, 136, 108))),
        (10, 74, 46, Some((1, 75, 47))),
        (1, 50, 22, Some((15, 51, 23))),
        (2, 42, 14, Some((17, 43, 15))),
    ],
    [
        // version 18
        (5, 150, 120, Some((1, 151, 121))),
        (9, 69, 43, Some((4, 70, 44))),
        (17, 50, 22, Some((1, 51, 23))),
        (2, 42, 14, Some((19, 43, 15))),
    ],
    [
        // version 19
        (3, 141, 113, Some((4, 142, 114))),
        (3, 70, 44, Some((11, 71, 45))),
        (17, 47, 21, Some((4, 48, 22))),
        (9, 39, 13, Some((16, 40, 14))),
    ],
    [
        // version 20
        (3, 135, 107, Some((5, 136, 108))),
        (3, 67, 41, Some((13, 68, 42))),
        (15, 54, 24, Some((5, 55, 25))),
        (15, 43, 15, Some((10, 44, 16))),
    ],
    [
        // version 21
        (4, 144, 116, Some((4, 145, 117))),
        (17, 68, 42, None),
        (17, 50, 22, Some((6, 51, 23))),
        (19, 46, 16, Some((6, 47, 17))),
    ],
    [
        // version 22
        (2, 139, 111, Some((7, 140, 112))),
        (17, 74, 46, None),
        (7, 54, 24, Some((16, 55, 25))),
        (34, 37, 13, None),
    ],
    [
        // version 23
        (4, 151, 121, Some((5, 152, 122))),
        (4, 75, 47, Some((14, 76, 48))),
        (11, 54, 24, Some((14, 55, 25))),
        (16, 45, 15, Some((14, 46, 16))),
    ],
    [
        // version 24
        (6, 147, 117, Some((4, 148, 118))),
        (6, 73, 45, Some((14, 74, 46))),
        (11, 54, 24, Some((16, 55, 25))),
        (30, 46, 16, Some((2, 47, 17))),
    ],
    [
        // version 25
        (8, 132, 106, Some((4, 133, 107))),
        (8, 75, 47, Some((13, 76, 48))),
        (7, 54, 24, Some((22, 55, 25))),
        (22, 45, 15, Some((13, 46, 16))),
    ],
    [
        // version 26
        (10, 142, 114, Some((2, 143, 115))),
        (19, 74, 46, Some((4, 75, 47))),
        (28, 50, 22, Some((6, 51, 23))),
        (33, 46, 16, Some((4, 47, 17))),
    ],
    [
        // version 27
        (8, 152, 122, Some((4, 153, 123))),
        (22, 73, 45, Some((3, 74, 46))),
        (8, 53, 23, Some((26, 54, 24))),
        (12, 45, 15, Some((28, 46, 16))),
    ],
    [
        // version 28
        (3, 147, 117, Some((10, 148, 118))),
        (3, 73, 45, Some((23, 74, 46))),
        (4, 54, 24, Some((31, 55, 25))),
        (11, 45, 15, Some((31, 46, 16))),
    ],
    [
        // version 29
        (7, 146, 116, Some((7, 147, 117))),
        (21, 73, 45, Some((7, 74, 46))),
        (1, 53, 23, Some((37, 54, 24))),
        (19, 45, 15, Some((26, 46, 16))),
    ],
    [
        // version 30
        (5, 145, 115, Some((10, 146, 116))),
        (19, 75, 47, Some((10, 76, 48))),
        (15, 54, 24, Some((25, 55, 25))),
        (23, 45, 15, Some((25, 46, 16))),
    ],
    [
        // version 31
        (13, 145, 115, Some((3, 146, 116))),
        (2, 74, 46, Some((29, 75, 47))),
        (42, 54, 24, Some((1, 55, 25))),
        (23, 45, 15, Some((28, 46, 16))),
    ],
    [
        // version 32
        (17, 145, 115, None),
        (10, 74, 46, Some((23, 75, 47))),
        (10, 54, 24, Some((35, 55, 25))),
        (19, 45, 15, Some((35, 46, 16))),
    ],
    [
        // version 33
        (17, 145, 115, Some((1, 146, 116))),
        (14, 74, 46, Some((21, 75, 47))),
        (29, 54, 24, Some((19, 55, 25))),
        (11, 45, 15, Some((46, 46, 16))),
    ],
    [
        // version 34
        (13, 145, 115, Some((6, 146, 116))),
        (14, 74, 46, Some((23, 75, 47))),
        (44, 54, 24, Some((7, 55, 25))),
        (59, 46, 16, Some((1, 47, 17))),
    ],
    [
        // version 35
        (12, 151, 121, Some((7, 152, 122))),
        (12, 75, 47, Some((26, 76, 48))),
        (39, 54, 24, Some((14, 55, 25))),
        (22, 45, 15, Some((41, 46, 16))),
    ],
    [
        // version 36
        (6, 151, 121, Some((14, 152, 122))),
        (6, 75, 47, Some((34, 76, 48))),
        (46, 54, 24, Some((10, 55, 25))),
        (2, 45, 15, Some((64, 46, 16))),
    ],
    [
        // version 37
        (17, 152, 122, Some((4, 153, 123))),
        (29, 74, 46, Some((14, 75, 47))),
        (49, 54, 24, Some((10, 55, 25))),
        (24, 45, 15, Some((46, 46, 16))),
    ],
    [
        // version 38
        (4, 152, 122, Some((18, 153, 123))),
        (13, 74, 46, Some((32, 75, 47))),
        (48, 54, 24, Some((14, 55, 25))),
        (42, 45, 15, Some((32, 46, 16))),
    ],
    [
        // version 39
        (20, 147, 117, Some((4, 148, 118))),
        (40, 75, 47, Some((7, 76, 48))),
        (43, 54, 24, Some((22, 55, 25))),
        (10, 45, 15, Some((67, 46, 16))),
    ],
    [
        // version 40
        (19, 148, 118, Some((6, 149, 119))),
        (18, 75, 47, Some((31, 76, 48))),
        (34, 54, 24, Some((34, 55, 25))),
        (20, 45, 15, Some((61, 46, 16))),
    ],
];

#[test]
fn ec_block_tests() {
    // ec block count, total codewords per block, data codewords per block
    // test that the no. of codewords add up,
    // that the no. of error-correcting codewords is increasing,
    // and that the optional blocks have a larger number of codewords
    for (index, &a) in EC_BLOCK_TABLE.iter().enumerate() {
        let codeword_total = CODEWORDS[index] as usize;
        let mut last_ecwords = 0;
        for (ec_lvl, &contents) in a.iter().enumerate() {
            let (block_count, cwords, dcwords, optional) = contents;
            if let Some((block_count_2, cwords_2, dcwords_2)) = optional {
                // codewords match
                assert!(
                    block_count * cwords + block_count_2 * cwords_2 == codeword_total,
                    "version {}, error correction {} - codeword count mismatch",
                    index + 1,
                    ec_lvl
                );

                // ec codewords match
                assert!(
                    ERROR_CORRECTION_CODEWORDS.contains(&((cwords - dcwords) as u32))
                        && ERROR_CORRECTION_CODEWORDS.contains(&((cwords_2 - dcwords_2) as u32)),
                    "version {}, error correction {} - ec codeword count mismatch",
                    index + 1,
                    ec_lvl
                );

                // ec codewords increasing
                assert!(
                    last_ecwords
                        < block_count * (cwords - dcwords) + block_count_2 * (cwords_2 - dcwords_2),
                    "version {}, error correction {} - ec codewords not monotonic",
                    index + 1,
                    ec_lvl
                );
                last_ecwords =
                    block_count * (cwords - dcwords) + block_count_2 * (cwords_2 - dcwords_2);

                // optional block bigger
                assert!(
                    cwords < cwords_2,
                    "version {}, error correction {} - optional block too small",
                    index + 1,
                    ec_lvl
                );

                // number of ec codewords equal between blocks
                assert!(
                    cwords - dcwords == cwords_2 - dcwords_2,
                    "version {}, error correction {} - error-correcting codewords not equal",
                    index + 1,
                    ec_lvl
                );
            } else {
                // codewords match
                assert!(
                    block_count * cwords == codeword_total,
                    "version {}, error correction {} - codeword count mismatch",
                    index + 1,
                    ec_lvl
                );

                // ec codewords match
                assert!(
                    ERROR_CORRECTION_CODEWORDS.contains(&((cwords - dcwords) as u32)),
                    "version {}, error correction {} - ec codeword count mismatch",
                    index + 1,
                    ec_lvl
                );

                // ec codewords increasing
                assert!(
                    last_ecwords < block_count * (cwords - dcwords),
                    "version {}, error correction {} - ec codewords not monotonic",
                    index + 1,
                    ec_lvl
                );
                last_ecwords = block_count * (cwords - dcwords);
            }
        }
    }
}

pub fn get_block_info(version: u32, level: u8) -> VersionBlockInfo {
    assert!(
        (1..=40).contains(&version) && level < 4,
        "incorrect version request"
    );
    EC_BLOCK_TABLE[version as usize - 1][level as usize]
}

// available data codewords per level and version
pub const DATA_CODEWORDS: [[usize; 40]; 4] = [
    [
        19, 34, 55, 80, 108, 136, 156, 194, 232, 274, 324, 370, 428, 461, 523, 589, 647, 721, 795,
        861, 932, 1006, 1094, 1174, 1276, 1370, 1468, 1531, 1631, 1735, 1843, 1955, 2071, 2191,
        2306, 2434, 2566, 2702, 2812, 2956,
    ],
    [
        16, 28, 44, 64, 86, 108, 124, 154, 182, 216, 254, 290, 334, 365, 415, 453, 507, 563, 627,
        669, 714, 782, 860, 914, 1000, 1062, 1128, 1193, 1267, 1373, 1455, 1541, 1631, 1725, 1812,
        1914, 1992, 2102, 2216, 2334,
    ],
    [
        13, 22, 34, 48, 62, 76, 88, 110, 132, 154, 180, 206, 244, 261, 295, 325, 367, 397, 445,
        485, 512, 568, 614, 664, 718, 754, 808, 871, 911, 985, 1033, 1115, 1171, 1231, 1286, 1354,
        1426, 1502, 1582, 1666,
    ],
    [
        9, 16, 26, 36, 46, 60, 66, 86, 100, 122, 140, 158, 180, 197, 223, 253, 283, 313, 341, 385,
        406, 442, 464, 514, 538, 596, 628, 661, 701, 745, 793, 845, 901, 961, 986, 1054, 1096,
        1142, 1222, 1276,
    ],
];

/* // available data codewords, followed by version and level
pub const INVERSE_DATA_CODEWORDS: [(u32, (u32, u32)); 160] = [
    (9, (1, 3)),
    (13, (1, 2)),
    (16, (1, 1)),
    (16, (2, 3)),
    (19, (1, 0)),
    (22, (2, 2)),
    (26, (3, 3)),
    (28, (2, 1)),
    (34, (2, 0)),
    (34, (3, 2)),
    (36, (4, 3)),
    (44, (3, 1)),
    (46, (5, 3)),
    (48, (4, 2)),
    (55, (3, 0)),
    (60, (6, 3)),
    (62, (5, 2)),
    (64, (4, 1)),
    (66, (7, 3)),
    (76, (6, 2)),
    (80, (4, 0)),
    (86, (5, 1)),
    (86, (8, 3)),
    (88, (7, 2)),
    (100, (9, 3)),
    (108, (5, 0)),
    (108, (6, 1)),
    (110, (8, 2)),
    (122, (10, 3)),
    (124, (7, 1)),
    (132, (9, 2)),
    (136, (6, 0)),
    (140, (11, 3)),
    (154, (8, 1)),
    (154, (10, 2)),
    (156, (7, 0)),
    (158, (12, 3)),
    (180, (11, 2)),
    (180, (13, 3)),
    (182, (9, 1)),
    (194, (8, 0)),
    (197, (14, 3)),
    (206, (12, 2)),
    (216, (10, 1)),
    (223, (15, 3)),
    (232, (9, 0)),
    (244, (13, 2)),
    (253, (16, 3)),
    (254, (11, 1)),
    (261, (14, 2)),
    (274, (10, 0)),
    (283, (17, 3)),
    (290, (12, 1)),
    (295, (15, 2)),
    (313, (18, 3)),
    (324, (11, 0)),
    (325, (16, 2)),
    (334, (13, 1)),
    (341, (19, 3)),
    (365, (14, 1)),
    (367, (17, 2)),
    (370, (12, 0)),
    (385, (20, 3)),
    (397, (18, 2)),
    (406, (21, 3)),
    (415, (15, 1)),
    (428, (13, 0)),
    (442, (22, 3)),
    (445, (19, 2)),
    (453, (16, 1)),
    (461, (14, 0)),
    (464, (23, 3)),
    (485, (20, 2)),
    (507, (17, 1)),
    (512, (21, 2)),
    (514, (24, 3)),
    (523, (15, 0)),
    (538, (25, 3)),
    (563, (18, 1)),
    (568, (22, 2)),
    (589, (16, 0)),
    (596, (26, 3)),
    (614, (23, 2)),
    (627, (19, 1)),
    (628, (27, 3)),
    (647, (17, 0)),
    (661, (28, 3)),
    (664, (24, 2)),
    (669, (20, 1)),
    (701, (29, 3)),
    (714, (21, 1)),
    (718, (25, 2)),
    (721, (18, 0)),
    (745, (30, 3)),
    (754, (26, 2)),
    (782, (22, 1)),
    (793, (31, 3)),
    (795, (19, 0)),
    (808, (27, 2)),
    (845, (32, 3)),
    (860, (23, 1)),
    (861, (20, 0)),
    (871, (28, 2)),
    (901, (33, 3)),
    (911, (29, 2)),
    (914, (24, 1)),
    (932, (21, 0)),
    (961, (34, 3)),
    (985, (30, 2)),
    (986, (35, 3)),
    (1000, (25, 1)),
    (1006, (22, 0)),
    (1033, (31, 2)),
    (1054, (36, 3)),
    (1062, (26, 1)),
    (1094, (23, 0)),
    (1096, (37, 3)),
    (1115, (32, 2)),
    (1128, (27, 1)),
    (1142, (38, 3)),
    (1171, (33, 2)),
    (1174, (24, 0)),
    (1193, (28, 1)),
    (1222, (39, 3)),
    (1231, (34, 2)),
    (1267, (29, 1)),
    (1276, (25, 0)),
    (1276, (40, 3)),
    (1286, (35, 2)),
    (1354, (36, 2)),
    (1370, (26, 0)),
    (1373, (30, 1)),
    (1426, (37, 2)),
    (1455, (31, 1)),
    (1468, (27, 0)),
    (1502, (38, 2)),
    (1531, (28, 0)),
    (1541, (32, 1)),
    (1582, (39, 2)),
    (1631, (29, 0)),
    (1631, (33, 1)),
    (1666, (40, 2)),
    (1725, (34, 1)),
    (1735, (30, 0)),
    (1812, (35, 1)),
    (1843, (31, 0)),
    (1914, (36, 1)),
    (1955, (32, 0)),
    (1992, (37, 1)),
    (2071, (33, 0)),
    (2102, (38, 1)),
    (2191, (34, 0)),
    (2216, (39, 1)),
    (2306, (35, 0)),
    (2334, (40, 1)),
    (2434, (36, 0)),
    (2566, (37, 0)),
    (2702, (38, 0)),
    (2812, (39, 0)),
    (2956, (40, 0)),
];
 */