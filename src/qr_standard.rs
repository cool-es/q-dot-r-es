//! Bitmap operations related to the QR standard.

use crate::{image, rdsm::galois};

/// Low-level encoding of binary streams.
pub mod badstream;
/// High-level encoding of characters.
pub mod bitstream;
/// Lookup tables specific to the QR standard.
pub mod tables;

/// Return `false` only for a valid QR code version (`1..=40`).
#[inline]
fn bad_version(version: u32) -> bool {
    !(1..=40).contains(&version)
}

// Return the version of a QR code based on its width.
#[inline]
fn size_to_version(size: usize) -> Option<u32> {
    if size % 4 == 1 && (21..=177).contains(&size) {
        Some((size as u32 - 17) / 4)
    } else {
        None
    }
}

/// Return `w - 1`, where `w` is the width of a version of QR code.
fn version_to_max_index(version: u32) -> Option<usize> {
    if bad_version(version) {
        None
    } else {
        Some(20 + 4 * (version as usize - 1))
    }
}

/// Is `(x, y)` a valid coordinate in a certain QR code?
fn out_of_bounds(x: usize, y: usize, version: u32) -> bool {
    if bad_version(version) {
        true
    } else {
        // x.max(y) + 1 > 21 + 4 * (version as usize - 1)
        // note that this is a comparison that returns bool
        (16 + 4 * version as usize) < x.max(y)
    }
}

/// Methods specific to the QR standard.
impl image::Bitmap {
    /// Apply a QR masking pattern to the image.
    pub fn qr_mask_xor(&mut self, pattern: u8) {
        qr_mask_xor(self, pattern)
    }

    /// Calculate the penalty score incurred by a certain masking pattern.
    pub fn qr_penalty(&self) -> u32 {
        penalties::total_penalty(self)
    }

    /// Return the QR code version (if any) based on the bitmap's dimensions.
    pub fn qr_version(&self) -> Option<u32> {
        let (x, y) = self.dims();
        if x != y {
            None
        } else {
            size_to_version(x)
        }
    }

    /// Create a blank QR code template.
    pub fn new_blank_qr(version: u32) -> Self {
        new_blank_qr_code(version)
    }
}

fn qr_mask_xor(input: &mut image::Bitmap, mask: u8) {
    let maybe_version = {
        if input.dims().0 != input.dims().1 {
            None
        } else {
            size_to_version(input.dims().0)
        }
    };

    for vec_index in 0..input.debug_bits().len() {
        let mut mask_byte = 0;
        for bit_index in (0..8).rev() {
            mask_byte <<= 1;
            if let Some((x, y)) = input.debug_index_to_xy(vec_index, bit_index) {
                if maybe_version.is_none() || coord_is_data(x, y, maybe_version.unwrap_or_default())
                {
                    mask_byte |= u8::from(
                        0 == match mask {
                            0 => (x + y) % 2,
                            1 => y % 2,
                            2 => x % 3,
                            3 => (x + y) % 3,
                            4 => (x / 3 + y / 2) % 2,
                            5 => (x * y) % 2 + (x * y) % 3,
                            6 => ((x * y) % 3 + x * y) % 2,
                            7 => ((x * y) % 3 + x + y) % 2,
                            _ => panic!(),
                        },
                    );
                }
            }
        }
        input.debug_bits_mut()[vec_index] ^= mask_byte;
    }
}

mod penalties {
    // Calculate the total penalty.
    pub fn total_penalty(input: &crate::image::Bitmap) -> u32 {
        let width = input.dims().0;
        let ones = input.debug_bits().iter().map(|x| x.count_ones()).sum();

        let size = usize::BITS as usize;
        let bit = {
            let mut bit_vector: Vec<usize> = Vec::new();
            let mut ticker = 0usize;
            let mut pushy = 0usize;

            for x in 0..width {
                for y in 0..width {
                    pushy |=
                        usize::from(input.get_bit(x, y).expect("out of bounds")) << (ticker % size);
                    ticker += 1;
                    if ticker % size == 0 {
                        bit_vector.push(pushy);
                        pushy = 0;
                    }
                }
            }
            bit_vector.push(pushy);
            bit_vector
        };

        // replicates .get_bit(), only very optimized
        // this function is called approximately 100 million times
        // when generating a version 40 code ...
        let get = |x, y| {
            let index = x * width + y;
            bit[index / size] & (1usize << (index % size)) != 0
        };

        let adjacent = adjacent(width, get);
        let block = block(width, get);
        let fake_marker = fake_marker(width, get);
        let proportion = proportion(width, ones);

        #[cfg(feature = "demo")]
        {
            // copy penalties to info struct
            crate::demo::ops::with_info(|info| {
                info.penalties
                    .copy_from_slice(&[adjacent, block, fake_marker, proportion])
            });
        }

        adjacent + block + fake_marker + proportion
    }

    // Penalty: "Adjacent modules in row/column in same color".
    // penalty: `3 + i`, where i is the amount by which the number of adjacent modules of the same color exceeds 5.
    fn adjacent<F>(width: usize, get: F) -> u32
    where
        F: Fn(usize, usize) -> bool,
    {
        let max = width - 1;

        let mut penalty = 0;
        for line in 0..=max {
            // get row n
            let (mut xrun, mut yrun) = (1, 1);
            for index in 0..=max {
                // checking x
                if index < max && (get(index, line) == get(index + 1, line)) {
                    xrun += 1;
                } else {
                    if xrun > 5 {
                        // 3 + run - 5
                        penalty += xrun - 2;
                    }
                    xrun = 1;
                }

                // checking y
                if index < max && get(line, index) == get(line, index + 1) {
                    yrun += 1;
                } else {
                    if yrun > 5 {
                        // 3 + run - 5
                        penalty += yrun - 2;
                    }
                    yrun = 1;
                }
            }
        }
        penalty as u32
    }

    /// penalty: `3 * (m - 1) * (n - 1)`
    /// where the block size = `m * n`
    fn block<F>(width: usize, get: F) -> u32
    where
        F: Fn(usize, usize) -> bool,
    {
        let max = width - 1;

        // to create a version 40 code, the
        // function get() is called, approximately,
        // NINETY EIGHT MILLION TIMES !!!

        // look for rectangles width (width of symbol), ... , 2
        // by using a sliding frame, and mark already-scored pixels.
        // this is no panacea, but it's an okay solution
        let mut penalty = 0;

        let mut scored = Vec::new();
        scored.resize(width.pow(2), false);

        for rect_width in (1..=max).rev() {
            // "leeway" is the range of acceptable starting values for x
            let leeway = max - rect_width;
            /*
            start traversing the bitmap row by row. skip forward to the end of a "failed rectangle", and skip to the next row if x then is > leeway

            if a series of similar pixels is found, check next line. if there is no match, continue checking at end of the series on the previous line. if there is a match, see how long it goes for, add the score and then mark all of the rectangle's pixels as scored
            */

            for y in 0..=(max - 1) {
                'row: for starting_x in 0..=leeway {
                    if scored[starting_x * width + y]
                        || get(starting_x, y) != get(starting_x, y + 1)
                    {
                        // already scored, or can't be a valid rectangle
                        continue;
                    }
                    /*
                    since we're looking for the widest rectangles first, there's no chance of the "discovery loop" breaking by hitting an already-scored pixel going sideways - that can only happen while going downwards. going sideways, the "discovery loop" will only ever be broken by hitting either a pixel of the other color or the side of the pattern
                    */
                    let color = get(starting_x, y);
                    for x_offset in 0..=rect_width {
                        let x = starting_x + x_offset;

                        // failure conditions, all of which
                        // make it a non-scoring pattern
                        if (get(x, y) != color)
                            || (get(x, y + 1) != color)
                            || scored[x * width + y + 1]
                        {
                            if x > leeway {
                                break 'row;
                            } else {
                                continue 'row;
                            }
                        }
                    }
                    // we are in a valid 2-row rectangle (at least)
                    // now to keep checking!
                    let mut rect_height = 1;

                    // extend rectangle downwards as far as possible
                    'extend: for y2 in (y + 2)..=max {
                        for x2 in starting_x..=(starting_x + rect_width) {
                            if (get(x2, y2) != color) || scored[x2 * width + y2] {
                                break 'extend;
                            }
                        }
                        rect_height += 1;
                    }

                    // mark rectangle's pixels as scored
                    for i in y..=(y + rect_height) {
                        for j in starting_x..=(starting_x + rect_width) {
                            scored[j * width + i] = true;
                        }
                    }

                    penalty += 3 * rect_width * rect_height;
                }
            }
        }
        penalty as u32
    }

    /// 1:1:3:1:1 ratio (dark:light:dark:light:dark) pattern in row/column.
    /// named "fake marker" because it can be confused with the position markers
    fn fake_marker<F>(width: usize, get: F) -> u32
    where
        F: Fn(usize, usize) -> bool,
    {
        let max = width - 1;

        // penalty: 40
        let mut penalty = 0;
        let pattern = 0b1011101;

        for line in 0..=max {
            for index in 0..=(max - 6) {
                'horizontal_test: {
                    for bit in 0..=6 {
                        if get(index + bit, line) != (pattern & (1 << bit) != 0) {
                            break 'horizontal_test;
                        }
                    }
                    // matching pattern
                    penalty += 40;
                }
                'vertical_test: {
                    for bit in 0..=6 {
                        if get(line, index + bit) != (pattern & (1 << bit) != 0) {
                            break 'vertical_test;
                        }
                    }
                    penalty += 40;
                }
            }
        }
        penalty
    }

    // #[allow(unused_variables)]
    /// Proportion of dark modules in entire symbol
    fn proportion(width: usize, ones: u32) -> u32 {
        // penalty: 10 * k
        // k is the rating of the deviation of the proportion of dark modules in the symbol from 50% in steps of 5%

        // this works fine assuming there's no extra "inaccessible" bits outside of
        // the bitmap's graphical boundary
        let area = width.pow(2);
        let proportion: f32 = (ones as f32) / (area as f32);

        10 * ((10.0 - 20.0 * proportion).abs().round() as u32)
    }
}

/// raw data for format writing/reading operations.
///
/// format data in a qr symbol is replicated in two positions.
/// this function gives pairs of coordinates `(x1, y1)`, `(x2, y2)`
/// relative to top left module of the finder pattern
/// from LSB (0) to MSB (14) (see pg. 60).
fn format_info_coords(version: u32, bit: u32) -> Option<((usize, usize), (usize, usize))> {
    if bad_version(version) || bit > 14 {
        // undefined
        return None;
    }

    // max offset from origin: width - 1
    let max = (16 + 4 * version) as usize;
    let bit = bit as usize;

    let coord1 = match bit {
        0..=5 => (8, bit),
        6..=7 => (8, bit + 1),
        8 => (7, 8),
        _ => (14 - bit, 8),
    };
    let coord2 = match bit {
        0..=7 => (max - bit, 8),
        _ => (8, bit + (max - 14)),
    };

    Some((coord1, coord2))
}

// (`fmt * 2^10` + remainder of `(fmt * 2^10) / g)` - this always has remainder 0.
// this works since all numbers in a galois field are their own additive inverse,
// and since (remainder of (k + remainder of k)) ==(remainder of k + remainder of k).
/// Generate a QR code's 15-bit format code.
pub fn data_to_fcode(correction_level: u8, mask_pattern: u8) -> Option<u16> {
    let fmt = (correction_level << 3) | mask_pattern;
    if fmt >= 32 || correction_level > 3 || mask_pattern > 7 {
        return None;
    }

    let fcode = (fmt as u32) << 10;
    let code = fcode as u16 | galois::qr_fcode_remainder(fcode) as u16;

    #[cfg(feature = "demo")]
    {
        crate::demo::ops::with_info(|x| x.format_info = code.to_be_bytes())
    }

    Some(code)
}

pub fn set_fcode(input: &mut image::Bitmap, version: u32, fcode: u16) {
    let mask = 0b0101_0100_0001_0010u16;

    for bit in 0..=14 {
        let ((x1, y1), (x2, y2)) = format_info_coords(version, bit).unwrap();
        let value = (fcode ^ mask) & (1 << bit) != 0;
        input.set_bit(x1, y1, value);
        input.set_bit(x2, y2, value);
    }
}

pub fn next_data_bit(x: usize, y: usize, version: u32) -> Option<(usize, usize)> {
    // naive, slow, and robust implementation of "next data bit"
    // simply zigzag as if the pattern was blank,
    // returning the next valid coord

    if !coord_is_data(x, y, version) {
        return None;
    }

    let max = version_to_max_index(version)?;
    let (mut x, mut y) = (x, y);

    // upper bound to avoid infinite loops
    for _i in 0..(max + 1).pow(2) {
        // x coord is on the right-hand side of a codeword
        // if x < 6, x needs to be odd; otherwise even
        if (x % 2 == 0) ^ (x < 6) {
            // ←
            (x, y) = (x - 1, y);
        } else {
            // is the codeword being read from bottom to top (negative y direction)?
            let up_codeword = (((max - x) / 2) % 2 == 0) ^ (x < 6);

            if (y == 0 && up_codeword) || (y == max && !up_codeword) {
                (x, y) = (x - 1, y);
            } else if up_codeword {
                (x, y) = (x + 1, y - 1);
            } else {
                (x, y) = (x + 1, y + 1);
            }
        }

        if x == 6 {
            x -= 1;
        }

        if coord_is_data(x, y, version) {
            break;
        }
        if (x, y) == (0, max) {
            return None;
        }
    }

    Some((x, y))
}

fn coord_is_alignment_pattern(x: usize, y: usize, version: u32) -> bool {
    if out_of_bounds(x, y, version) {
        panic!("x = {}, y = {}", x, y)
    }

    let indices = tables::AP_COORD_INDICES[version as usize - 1];
    for (h, &hc) in indices.iter().enumerate() {
        if x.abs_diff(hc) < 3 {
            for (v, &vc) in indices.iter().enumerate() {
                if y.abs_diff(vc) < 3 {
                    // making sure not to include the non-existent alignment patterns
                    if h.max(v) == 0 || (h.min(v) == 0 && h.max(v) == indices.len() - 1) {
                        break;
                    }
                    return true;
                }
            }
            break;
        }
    }
    false
}

#[inline]
pub fn coord_is_data(x: usize, y: usize, version: u32) -> bool {
    coord_status(x, y, version).is_some_and(|c| c == 0)
}

/// returns the type of pixel taken up by a coordinate in a qr code.
///
/// from 0 to 5: data, position, timing, format, alignment, version, that one bit
pub fn coord_status(x: usize, y: usize, version: u32) -> Option<u8> {
    if out_of_bounds(x, y, version) {
        return None;
    }

    Some(if coord_is_alignment_pattern(x, y, version) {
        // alignment pattern
        4
    } else if x < 8 && y < 8 {
        // top left position square
        1
    } else {
        let max = version_to_max_index(version)?;
        if (x <= 7 && max - y <= 7) || (y <= 7 && max - x <= 7) {
            // other two position squares
            1
        } else if x == 6 || y == 6 {
            // timing pattern
            2
        } else if (x == 8 && (y <= 8 || max - y <= 6)) || (y == 8 && (x <= 8 || max - x <= 7)) {
            // format pattern
            3
        } else if (x, y) == (8, max - 7) {
            // singular constant bit that's always 1
            6
        } else if version >= 7 && x.min(y) <= 5 && x.max(y) >= max - 10 {
            // version pattern
            5
        } else {
            // data
            0
        }
    })
}

fn new_blank_qr_code(version: u32) -> image::Bitmap {
    let max = version_to_max_index(version).expect("invalid version");
    let mut output = image::Bitmap::new(max + 1, max + 1);
    let mut set = |x, y| output.set_bit(x, y, true);

    //  draw alignment patters
    let alignment_coords = tables::alignment_pattern_coords(version);
    for (x, y) in alignment_coords {
        set(x, y);
        for i in 0..=3 {
            // draw it in a rotationally symmetric way, clockwise
            set((x - 1) + i, y - 2); // top
            set(x + 2, (y - 1) + i); // right
            set((x + 1) - i, y + 2); // bottom
            set(x - 2, (y + 1) - i); // left
        }
    }

    // draw timing patterns
    for i in 8..=(max - 8) {
        if i % 2 == 0 {
            set(i, 6);
            set(6, i);
        }
    }

    // draw position patterns
    for x in 0..=6usize {
        for y in 0..=6usize {
            if (x.abs_diff(3) == 2 && y.abs_diff(3) != 3)
                || (y.abs_diff(3) == 2 && x.abs_diff(3) != 3)
            {
                // the white ring around the center of the position pattern
                continue;
            }
            set(x, y);
            set(y, max - x);
            set(max - y, x);
        }
    }

    // draw the singular black bit
    set(8, max - 7);

    // draw version patterns
    if version >= 7 {
        set_vcode(&mut output, version, qr_generate_vcode(version));
    }

    #[cfg(feature = "demo")]
    {
        // copy bitmap to info structure
        crate::demo::ops::set_bitmap(&output, |info| &mut info.bitmap_base);
    }

    output
}

// tested, works!
/// generate the 18-bit version info data (versions 7 and up)
fn qr_generate_vcode(version: u32) -> u32 {
    // version code generator for (18,6) BCH code:
    // 0x1F25 = 0b1111100100101
    ((version << 12) | galois::carryless_divide(version << 12, 0x1F25)) as u32
}

// in the style of format_info_coords. again:
/// this function gives pairs of coordinates `(x1, y1)`, `(x2, y2)`
/// relative to top left module of the finder pattern
/// from LSB (0) to MSB (17) (see pg. 61)
fn version_info_coords(version: u32, bit: u32) -> Option<((usize, usize), (usize, usize))> {
    if bad_version(version) || version < 7 || bit > 17 {
        // undefined
        return None;
    }

    let max = version_to_max_index(version)?;
    let bit = bit as usize;

    let short = bit % 3 + max - 10;
    let long = bit / 3;

    // diagonal mirror symmetry
    let coord1 = (short, long);
    let coord2 = (long, short);

    Some((coord1, coord2))
}

// in the style of set_fcode
pub fn set_vcode(input: &mut image::Bitmap, version: u32, vcode: u32) {
    for bit in 0..=17 {
        let ((x1, y1), (x2, y2)) = version_info_coords(version, bit).expect("bad version");
        let value = vcode & (1 << bit) != 0;
        input.set_bit(x1, y1, value);
        input.set_bit(x2, y2, value);
    }
}
