// returns the standard sizes for qr code symbols, indexed by version number
// 21*21, 25*25, ..., 177*177
pub fn version_to_size(version: u32) -> Option<u32> {
    if !(1..=40).contains(&version) {
        None
    } else {
        Some(17 + 4 * version)
    }
}

// Img methods pertaining to the qr standard specifically
impl super::continuous::Img {
    // generates a new bitmap overlaid with a bit pattern - VERY inefficient
    // adapted from the code in qr/src/bitmask.rs
    // returns None if 'mask' isn't within the range of 0 to 7
    pub fn _new_qr_mask(w: usize, h: usize, pattern: u8) -> Option<Self> {
        let mut output = Self::new(w, h);

        for y in 0..h {
            for x in 0..w {
                output.set_bit(
                    x,
                    y,
                    match pattern {
                        0 => (x + y) % 2,
                        1 => y % 2,
                        2 => x % 3,
                        3 => (x + y) % 3,
                        4 => (x / 3 + y / 2) % 2,
                        5 => (x * y) % 2 + (x * y) % 3,
                        6 => ((x * y) % 3 + x * y) % 2,
                        7 => ((x * y) % 3 + x + y) % 2,
                        _ => return None,
                    } == 0,
                );
            }
        }
        Some(output)
    }

    // xor one of the qr masking patterns over the bitmap, directly
    // efficient, should replace _new_qr_mask():
    // _new_qr_mask(a, b, x) == new(a, b).qr_mask_xor(x)
    // i wrote this on the first try just before bedtime. go me
    pub fn qr_mask_xor(&mut self, pattern: u8) {
        for vec_index in 0..self.bits.len() {
            let mut mask_byte = 0u8;
            for bit_index in (0..8).rev() {
                mask_byte <<= 1;
                if let Some((x, y)) =
                    super::continuous::index_to_xy(vec_index, bit_index, self.width, self.height)
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
            self.bits[vec_index] ^= mask_byte;
        }
    }

    pub fn penalty(&self) -> u32 {
        // returns the qr code penalty for a bitmap, to choose xor patterns
        // 4 tests, weighted 3, 3, 40, 10
        /*
            Although the masking operation is only performed on the encoding region of the symbol excluding the Format Information, the area to be evaluated is the complete symbol. The Mask Pattern which results in the lowest score shall be selected for the symbol.
        */

        // Adjacent modules in row/column in same color
        let test1: u32 = {
            //  i is the amount by which the number of adjacent modules of the same color exceeds 5
            todo!()
        };

        // Block of modules in same color
        let test2: (u32, u32) = {
            // Block size = m×n
            todo!()
        };

        // 1:1:3:1:1 ratio (dark:light:dark:light:dark) pattern in row/column
        let test3: u32 = {
            // i'm very unsure if this is cumulative or just a binary...
            // this is meant to stop patterns that look like the qr alignment marks,
            // so i assume this must be cumulative - otherwise every qr code would be tied
            todo!()
        };

        // Proportion of dark modules in entire symbol
        let test4: u32 = {
            // k is the rating of the deviation of the proportion of dark modules in the symbol from 50% in steps of 5%
            todo!()
        };

        (3 + test1) + (3 * (test2.0 - 1) * (test2.1 - 1)) + (40 * test3) + (10 * test4)
    }
}
