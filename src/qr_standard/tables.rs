/// the centers of alignment patterns in both x and y dimensions
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

pub(crate) fn alignment_pattern_coords(version: u32) -> Vec<(usize, usize)> {
    // this function is only called by new_blank_qr_code,
    // which has checked the version number already - no
    // need to do it again
    let indices = AP_COORD_INDICES[version as usize - 1];
    let mut output = Vec::new();

    for &x in indices {
        for &y in indices {
            if [x, y].contains(&6) && (x == y || [x, y].contains(indices.last().unwrap())) {
                continue;
            }
            output.push((x, y));
        }
    }
    output
}

/// reverse-lookup to find the index for precomputed.rs > RDSM_GENERATOR_POLYNOMIALS
#[inline]
pub(crate) fn find_errc(input: usize) -> Option<usize> {
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

pub(super) const ALPHANUM_SET: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";

pub(crate) type VersionBlockInfo = (usize, usize, usize, Option<(usize, usize, usize)>);

#[doc(hidden)]
// generates up to 24 version block info points from 13 variables
// total_codewords: the number of codewords that fit in a code
// level_data: array of
//      error-correcting codewords per block,
//      number of blocks of type 1,
//      number of blocks of type 2
const fn _vbi(total_codewords: usize, level_data: [[usize; 3]; 4]) -> [VersionBlockInfo; 4] {
    let mut arr: [VersionBlockInfo; 4] = [(0, 0, 0, None); 4];
    let mut i = 0;

    // const functions can't use 'for' loops,
    // so use 'while'-and-increment instead
    while i < 4 {
        let [ecc, bc1, bc2] = level_data[i];
        assert!((total_codewords - bc2) % (bc1 + bc2) == 0);
        let cw1 = (total_codewords - bc2) / (bc1 + bc2);
        let dcw1 = cw1 - ecc;
        let opt = if bc2 == 0 {
            None
        } else {
            Some((bc2, cw1 + 1, dcw1 + 1))
        };

        arr[i] = (bc1, cw1, dcw1, opt);
        i += 1;
    }

    arr
}

/// error correction data (pg. 41...).
/// access with
/// ```
/// EC_BLOCK_TABLE[version-1][correction_level]
/// ```
/// correction levels are ordered L - M - Q - H.
/// data format is:
/// ec block count, total codewords per block, data codewords per block.
/// if there is just one block variant, the other pair member will be `None`.
const EC_BLOCK_TABLE: [[VersionBlockInfo; 4]; 40] = [
    _vbi(
        26, // version 1
        [[7, 1, 0], [10, 1, 0], [13, 1, 0], [17, 1, 0]],
    ),
    _vbi(
        44, // version 2
        [[10, 1, 0], [16, 1, 0], [22, 1, 0], [28, 1, 0]],
    ),
    _vbi(
        70, // version 3
        [[15, 1, 0], [26, 1, 0], [18, 2, 0], [22, 2, 0]],
    ),
    _vbi(
        100, // version 4
        [[20, 1, 0], [18, 2, 0], [26, 2, 0], [16, 4, 0]],
    ),
    _vbi(
        134, // version 5
        [[26, 1, 0], [24, 2, 0], [18, 2, 2], [22, 2, 2]],
    ),
    _vbi(
        172, // version 6
        [[18, 2, 0], [16, 4, 0], [24, 4, 0], [28, 4, 0]],
    ),
    _vbi(
        196, // version 7
        [[20, 2, 0], [18, 4, 0], [18, 2, 4], [26, 4, 1]],
    ),
    _vbi(
        242, // version 8
        [[24, 2, 0], [22, 2, 2], [22, 4, 2], [26, 4, 2]],
    ),
    _vbi(
        292, // version 9
        [[30, 2, 0], [22, 3, 2], [20, 4, 4], [24, 4, 4]],
    ),
    _vbi(
        346, // version 10
        [[18, 2, 2], [26, 4, 1], [24, 6, 2], [28, 6, 2]],
    ),
    _vbi(
        404, // version 11
        [[20, 4, 0], [30, 1, 4], [28, 4, 4], [24, 3, 8]],
    ),
    _vbi(
        466, // version 12
        [[24, 2, 2], [22, 6, 2], [26, 4, 6], [28, 7, 4]],
    ),
    _vbi(
        532, // version 13
        [[26, 4, 0], [22, 8, 1], [24, 8, 4], [22, 12, 4]],
    ),
    _vbi(
        581, // version 14
        [[30, 3, 1], [24, 4, 5], [20, 11, 5], [24, 11, 5]],
    ),
    _vbi(
        655, // version 15
        [[22, 5, 1], [24, 5, 5], [30, 5, 7], [24, 11, 7]],
    ),
    _vbi(
        733, // version 16
        [[24, 5, 1], [28, 7, 3], [24, 15, 2], [30, 3, 13]],
    ),
    _vbi(
        815, // version 17
        [[28, 1, 5], [28, 10, 1], [28, 1, 15], [28, 2, 17]],
    ),
    _vbi(
        901, // version 18
        [[30, 5, 1], [26, 9, 4], [28, 17, 1], [28, 2, 19]],
    ),
    _vbi(
        991, // version 19
        [[28, 3, 4], [26, 3, 11], [26, 17, 4], [26, 9, 16]],
    ),
    _vbi(
        1085, // version 20
        [[28, 3, 5], [26, 3, 13], [30, 15, 5], [28, 15, 10]],
    ),
    _vbi(
        1156, // version 21
        [[28, 4, 4], [26, 17, 0], [28, 17, 6], [30, 19, 6]],
    ),
    _vbi(
        1258, // version 22
        [[28, 2, 7], [28, 17, 0], [30, 7, 16], [24, 34, 0]],
    ),
    _vbi(
        1364, // version 23
        [[30, 4, 5], [28, 4, 14], [30, 11, 14], [30, 16, 14]],
    ),
    _vbi(
        1474, // version 24
        [[30, 6, 4], [28, 6, 14], [30, 11, 16], [30, 30, 2]],
    ),
    _vbi(
        1588, // version 25
        [[26, 8, 4], [28, 8, 13], [30, 7, 22], [30, 22, 13]],
    ),
    _vbi(
        1706, // version 26
        [[28, 10, 2], [28, 19, 4], [28, 28, 6], [30, 33, 4]],
    ),
    _vbi(
        1828, // version 27
        [[30, 8, 4], [28, 22, 3], [30, 8, 26], [30, 12, 28]],
    ),
    _vbi(
        1921, // version 28
        [[30, 3, 10], [28, 3, 23], [30, 4, 31], [30, 11, 31]],
    ),
    _vbi(
        2051, // version 29
        [[30, 7, 7], [28, 21, 7], [30, 1, 37], [30, 19, 26]],
    ),
    _vbi(
        2185, // version 30
        [[30, 5, 10], [28, 19, 10], [30, 15, 25], [30, 23, 25]],
    ),
    _vbi(
        2323, // version 31
        [[30, 13, 3], [28, 2, 29], [30, 42, 1], [30, 23, 28]],
    ),
    _vbi(
        2465, // version 32
        [[30, 17, 0], [28, 10, 23], [30, 10, 35], [30, 19, 35]],
    ),
    _vbi(
        2611, // version 33
        [[30, 17, 1], [28, 14, 21], [30, 29, 19], [30, 11, 46]],
    ),
    _vbi(
        2761, // version 34
        [[30, 13, 6], [28, 14, 23], [30, 44, 7], [30, 59, 1]],
    ),
    _vbi(
        2876, // version 35
        [[30, 12, 7], [28, 12, 26], [30, 39, 14], [30, 22, 41]],
    ),
    _vbi(
        3034, // version 36
        [[30, 6, 14], [28, 6, 34], [30, 46, 10], [30, 2, 64]],
    ),
    _vbi(
        3196, // version 37
        [[30, 17, 4], [28, 29, 14], [30, 49, 10], [30, 24, 46]],
    ),
    _vbi(
        3362, // version 38
        [[30, 4, 18], [28, 13, 32], [30, 48, 14], [30, 42, 32]],
    ),
    _vbi(
        3532, // version 39
        [[30, 20, 4], [28, 40, 7], [30, 43, 22], [30, 10, 67]],
    ),
    _vbi(
        3706, // version 40
        [[30, 19, 6], [28, 18, 31], [30, 34, 34], [30, 20, 61]],
    ),
];

pub(crate) fn get_block_info(version: u32, level: u8) -> VersionBlockInfo {
    assert!(
        (1..=40).contains(&version) && level < 4,
        "incorrect version request"
    );
    EC_BLOCK_TABLE[version as usize - 1][level as usize]
}

/// available data codewords per level and version
pub(crate) const DATA_CODEWORDS: [[usize; 40]; 4] = [
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

/// no. of bits in the char count indicator by size class (1.. - 10.. - 27..),
/// and by mode (num-aln-asc-knj)
pub(super) const CC_INDICATOR_BITS: [[usize; 4]; 3] =
    [[10, 9, 8, 8], [12, 11, 16, 10], [14, 13, 16, 12]];

pub(crate) fn cc_indicator_bit_size(class: u8, mode: super::Mode) -> usize {
    use super::Mode::*;
    if class < 3 {
        CC_INDICATOR_BITS[class as usize][match mode {
            Numeric => 0,
            AlphaNum => 1,
            ASCII => 2,
            Kanji => 3,
        }]
    } else {
        panic!("access out of bounds")
    }
}

#[inline]
pub(crate) const fn version_to_class(version: u32) -> u8 {
    match version {
        // no. of bits in char count indicator per version
        1..=9 => 0,
        10..=26 => 1,
        27..=40 => 2,
        _ => panic!(),
    }
}
