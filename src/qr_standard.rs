use super::*;

mod tables;
pub use tables::*;
mod bitstream;
pub use bitstream::*;

fn bad_version(version: u32) -> bool {
    !(1..=40).contains(&version)
}

// returns the standard sizes for qr code symbols, indexed by version number
// 21*21, 25*25, ..., 177*177
pub fn version_to_size(version: u32) -> Option<u32> {
    if bad_version(version) {
        None
    } else {
        Some(21 + 4 * (version - 1))
    }
}

fn size_to_version(size: usize) -> Option<u32> {
    if size % 4 == 1 && (21..=177).contains(&size) {
        Some((size as u32 - 17) / 4)
    } else {
        None
    }
}

fn version_to_max_index(version: u32) -> usize {
    if bad_version(version) {
        panic!()
    }
    20 + 4 * (version as usize - 1)
}

fn out_of_bounds(x: usize, y: usize, version: u32) -> bool {
    if bad_version(version) {
        true
    } else {
        // x.max(y) + 1 > 21 + 4 * (version as usize - 1)
        // note that this is a comparison that returns bool
        x.max(y) > 16 + 4 * version as usize
    }
}

// Img methods pertaining to the qr standard specifically
impl image::Img {
    pub fn qr_mask_xor(&mut self, pattern: u8) {
        qr_mask_xor(self, pattern)
    }
    pub fn qr_penalty(&self) -> u32 {
        penalty(self)
    }
    pub fn qr_version(&self) -> Option<u32> {
        use image::*;

        let (x, y) = self.dims();
        if x != y {
            None
        } else {
            size_to_version(x)
        }
    }
    pub fn new_blank_qr(version: u32) -> Self {
        new_blank_qr_code(version)
    }
}

// ImgRowAligned methods, ditto
impl image::ImgRowAligned {
    pub fn qr_mask_xor(&mut self, pattern: u8) {
        qr_mask_xor(self, pattern)
    }
    pub fn qr_penalty(&self) -> u32 {
        penalty(self)
    }
    pub fn qr_version(&self) -> Option<u32> {
        use image::*;

        let (x, y) = self.dims();
        if x != y {
            None
        } else {
            size_to_version(x)
        }
    }
    pub fn new_blank_qr(version: u32) -> Self {
        new_blank_qr_code(version)
    }
}

// xor one of the qr masking patterns over the bitmap, directly
// efficient, should replace _new_qr_mask():
// _new_qr_mask(a, b, x) == new(a, b).qr_mask_xor(x)
// i wrote this on the first try just before bedtime. go me
// modified to leave gaps in the pattern for valid qr version sizes
fn qr_mask_xor<T: image::Bitmap>(input: &mut T, pattern: u8) {
    let maybe_version = {
        if input.dims().0 != input.dims().1 {
            None
        } else {
            size_to_version(input.dims().0)
        }
    };

    for vec_index in 0..input.debug_bits().len() {
        let mut mask_byte = 0u8;
        for bit_index in (0..8).rev() {
            mask_byte <<= 1;
            if let Some((x, y)) = input.debug_index_to_xy(vec_index, bit_index) {
                if maybe_version.is_none() || coord_is_data(x, y, maybe_version.unwrap_or_default())
                {
                    mask_byte |= (match pattern {
                        0 => (x + y) % 2,
                        1 => y % 2,
                        2 => x % 3,
                        3 => (x + y) % 3,
                        4 => (x / 3 + y / 2) % 2,
                        5 => (x * y) % 2 + (x * y) % 3,
                        6 => ((x * y) % 3 + x * y) % 2,
                        7 => ((x * y) % 3 + x + y) % 2,
                        _ => panic!(),
                    } == 0) as u8;
                }
            }
        }
        input.debug_bits_mut()[vec_index] ^= mask_byte;
    }
}

#[allow(unused_variables, unreachable_code)]
fn penalty<T: image::Bitmap>(input: &T) -> u32 {
    // returns the qr code penalty for a bitmap, to choose xor patterns
    // 4 tests, weighted 3, 3, 40, 10
    /*
        Although the masking operation is only performed on the encoding region of the symbol excluding the Format Information, the area to be evaluated is the complete symbol. The Mask Pattern which results in the lowest score shall be selected for the symbol.
    */

    // i think surely, this can't be a question of an on/off thing... i guess it doesn't terribly matter since the code will scan anyway, even if the calculations are off
    // i've decided to keep going based on this, so this would mean that:
    // • every row or column with > 5 modules in it is penalized *individually*
    // • ... and every block is, as well
    // • ... and every 1:1:3:1:1 pattern is, as well
    // • ... and the pattern weight is penalized once

    // Adjacent modules in row/column in same color
    let test1: u32 = {
        // penalty: 3 + i
        //  i is the amount by which the number of adjacent modules of the same color exceeds 5

        let (dimension, _height) = input.dims();
        for n in 0..dimension {
            // get row n
            // use get_row here
            todo!()

            // get column n
        }
        todo!()
    };

    // Block of modules in same color
    let test2: (u32, u32) = {
        // penalty: 3 * (m - 1) * (n - 1)
        // where the block size = m * n
        todo!()
    };

    // 1:1:3:1:1 ratio (dark:light:dark:light:dark) pattern in row/column
    let test3: u32 = {
        // penalty: 40

        // i'm very unsure if this is cumulative or just a binary...
        // this is meant to stop patterns that look like the qr alignment marks,
        // so i assume this must be cumulative - otherwise every qr code would be tied
        todo!()
    };

    // Proportion of dark modules in entire symbol
    let test4: u32 = {
        // penalty: 10 * k
        // k is the rating of the deviation of the proportion of dark modules in the symbol from 50% in steps of 5%
        todo!()
    };

    todo!()
    // (3 + test1) + (3 * (test2.0 - 1) * (test2.1 - 1)) + (40 * test3) + (10 * test4)
}

// raw data for format writing/reading operations
// format data in a ~qr symbol~ is replicated in two positions:
// this function gives pairs of coordinates (x1, y1), (x2, y2)
// relative to top left module of the finder pattern
// from LSB (0) to MSB (14) (see pg. 60)
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

// ref. pg. 60
pub fn get_fcode<T: image::Bitmap>(input: &T, version: u32, offset: (usize, usize)) -> Option<u16> {
    // the coordinates of the top left module; in hellocode, it's (2,2)
    let (ox, oy) = offset;
    let mut output1 = 0;
    let mut output2 = 0;

    for bit in (0..=14).rev() {
        let ((x1, y1), (x2, y2)) = format_info_coords(version, bit)?;

        output1 <<= 1;
        output1 += u16::from(input.get_bit(x1 + ox, y1 + oy)?);

        output2 <<= 1;
        output2 += u16::from(input.get_bit(x2 + ox, y2 + oy)?);
    }

    if output1 != output2 {
        return None;
    }

    // mask value for format codes, 0x5412
    let mask = 0b0101_0100_0001_0010;

    Some(output1 ^ mask)
}

// returns error correction level and mask pattern (pg. 59)
pub fn interpret_format(fcode: u16) -> Option<(u8, u8)> {
    if !crate::rdsm::qr_fcode_is_good(fcode) {
        return None;
    }

    // L, M, Q, H
    // let mut correction = match 0b11 & (fcode >> 13) {
    //     0b01 => 1,
    //     0b00 => 2,
    //     0b11 => 3,
    //     0b10 | _ => 4,
    // };
    let correction = (0b11 & (fcode >> 13)) as u8;

    let maskpat = (0b111 & (fcode >> 10)) as u8;

    Some((correction, maskpat))
}

pub fn data_to_fcode(correction_level: u8, mask_pattern: u8) -> Option<u16> {
    if correction_level > 3 || mask_pattern > 7 {
        return None;
    }

    crate::rdsm::qr_generate_fcode((correction_level << 3) | mask_pattern)
}

pub fn set_fcode<T: image::Bitmap>(
    input: &mut T,
    version: u32,
    offset: (usize, usize),
    fcode: u16,
) {
    let (ox, oy) = offset;
    let mask = 0b0101_0100_0001_0010u16;

    for bit in 0..=14 {
        let ((x1, y1), (x2, y2)) = format_info_coords(version, bit).unwrap();
        let value = (fcode ^ mask) & (1 << bit) != 0;
        input.set_bit(x1 + ox, y1 + oy, value);
        input.set_bit(x2 + ox, y2 + oy, value);
    }
}

// return the coordinates of a given byte/codeword in a qr symbol (quite inefficiently)
// fn qr_data_coords(codeword: u32, bit: u8, version: u32) -> Option<(usize, usize)> {
//     let size = version_to_size(version)?;
//     let bitstream_index = codeword * 8 + bit as u32;

//     // only v1 for now
//     if version != 1 {
//         return None;
//     }

//     todo!()
// }

// returns the next bit of data after this one
pub fn _obsolete_next_data_bit(x: usize, y: usize, version: u32) -> Option<(usize, usize)> {
    // larger versions contain version information blocks which i haven't implemented yet
    if version > 6 {
        return None;
    }

    if !coord_is_data(x, y, version) {
        return None;
    }

    // x coord is on the right-hand side of a codeword
    // if x < 6, x needs to be odd; otherwise even
    if (x % 2 == 0) ^ (x < 6) {
        // ←
        return Some((x - 1, y));
    }

    let max = version_to_max_index(version);

    // the last bit in the pattern (assuming no version data!!)
    if x == 0 && (max - y) == 8 {
        return None;
    }

    // the only discontinuous part of the pattern - lower left corner
    if (x, y) == (9, max) {
        return Some((8, max - 8));
    }

    // the one skip over the vertical timing pattern
    if (x, y) == (7, 9) {
        return Some((5, 9));
    }

    // abbreviations
    let is_ap = |xi, yi| coord_is_alignment_pattern(xi, yi, version);

    // is the codeword being read from bottom to top (negative y direction)?
    let up_codeword = (((max - x) / 2) % 2 == 0) ^ (x < 6);

    if up_codeword {
        // about to hit top, go left
        if y == 0 {
            return Some((x - 1, y));
        }

        // about to hit top left / top right position markers
        if y == 9 && (x < 9 || (max - x) < 8) {
            // if x == 0 {
            //     return None;
            // }
            return Some((x - 1, y));
        }

        // about to hit timing pattern
        if y == 7 {
            return Some((x + 1, y - 2));
        }

        // about to hit an alignment pattern
        if is_ap(x, y - 1) {
            // we need to jump over it
            return Some((x + 1, y - 6));
        }
        if is_ap(x + 1, y - 1) {
            // we can sidle past it
            return Some((x, y - 1));
        }

        // nothing in the way. all is well
        return Some((x + 1, y - 1));
    } else {
        // about to hit bottom, go left
        if y == max {
            return Some((x - 1, y));
        }

        // about to hit lower left position marker
        if y == max - 8 && (x < 9) {
            // if x == 0 {
            //     return None;
            // }
            return Some((x - 1, y));
        }

        // about to hit timing pattern
        if y == 5 {
            return Some((x + 1, y + 2));
        }

        // about to hit alignment pattern
        if is_ap(x, y + 1) {
            // we need to jump over it
            return Some((x + 1, y + 6));
        }

        // nothing in the way
        return Some((x + 1, y + 1));
    }
}

pub fn next_data_bit(x: usize, y: usize, version: u32) -> Option<(usize, usize)> {
    // naive, slow, and robust implementation of "next data bit"
    // simply zigzag as if the pattern was blank,
    // returning the next valid coord

    if !coord_is_data(x, y, version) {
        return None;
    }

    let max = version_to_max_index(version);
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

    let indices = AP_COORD_INDICES[version as usize - 1];
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

pub fn coord_is_data(x: usize, y: usize, version: u32) -> bool {
    coord_status(x, y, version) == 0
}

// from 0 to 5:
// data, position, timing, format, alignment, version, that one bit
pub fn coord_status(x: usize, y: usize, version: u32) -> u8 {
    if out_of_bounds(x, y, version) {
        return u8::MAX;
    }

    if coord_is_alignment_pattern(x, y, version) {
        // alignment pattern
        4
    } else if x < 8 && y < 8 {
        // top left position square
        1
    } else {
        let max = version_to_max_index(version);
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
    }
}

fn new_blank_qr_code<T: image::Bitmap>(version: u32) -> T {
    let max = version_to_max_index(version);
    let mut output = T::new(max + 1, max + 1);
    let mut set = |x, y| output.set_bit(x as usize, y as usize, true);

    {
        //  draw alignment patters
        let alignment_coords = alignment_pattern_coords(version);
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
    }

    {
        // draw timing patterns
        for i in 8..=(max - 8) {
            if i % 2 == 0 {
                set(i, 6);
                set(6, i);
            }
        }
    }

    // draw position patterns
    {
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
    }

    output
}
