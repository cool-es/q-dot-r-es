use std::ops::BitXorAssign;

// 2-dimensional one bit data type, stored in a vector of u8's
pub struct Img {
    width: usize,
    height: usize,
    bits: Vec<u8>,
}

impl Img {
    pub fn new(w: usize, h: usize) -> Self {
        let mut vec: Vec<u8> = Vec::new();

        // resize vector to contain the maximum needed amount of
        // elements – that is, the index of the byte containing the last pixel
        // risk of fencepost error: resize() counts from 1, xy_to_index from 0
        vec.resize(xy_to_index(w - 1, h - 1, w, h).unwrap().0 + 1, 0);

        Img {
            width: w,
            height: h,
            bits: vec,
        }
    }

    // returns "false" if out-of-bounds
    pub fn set_bit(&mut self, x: usize, y: usize, bit: bool) -> bool {
        if let Some((n, i)) = xy_to_index(x, y, self.width, self.height) {
            if bit {
                // set a 1 (bitwise 'or' w/ 1)
                self.bits[n] |= 1 << i;
            } else {
                //set a 0 (bitwise 'and' w/ 0)
                self.bits[n] &= !(1 << i);
            }
            true
        } else {
            /* println!(
                "out-of-bounds write (x={} y={} w={} h={})",
                x, y, self.width, self.height
            ); */
            false
        }
    }

    pub fn get_bit(&self, x: usize, y: usize) -> Option<bool> {
        if let Some((n, i)) = xy_to_index(x, y, self.width, self.height) {
            Some(((self.bits[n] >> i) & 1) == 1)
        } else {
            None
        }
    }

    // apply 'pattern' to the image, but only the
    // bits specified by 'mask'
    pub fn mask_set(&mut self, pattern: &Img, mask: &Img) {
        if self.dims() != pattern.dims() || self.dims() != mask.dims() {
            // size mismatch
            panic!()
        }

        for i in 0..self.bits.len() {
            // (P & M) | (S & !M)
            // if M is 1, output is == P
            // if M is 0, output is == S
            self.bits[i] = (pattern.bits[i] & mask.bits[i]) | (self.bits[i] & !mask.bits[i]);
        }
    }

    //returns a given row as the bits of a u128
    pub fn get_row(&self, y: usize) -> u128 {
        if self.width > 128 {
            // won't fit
            // placeholder
            panic!()

            // maybe handle this by using an enum that could be different-size tuples of u128's?
        }

        if let Some((vec_index_start, bit_index_start)) = xy_to_index(0, y, self.width, self.height)
        {
            // the padding operation is going to be a bit tricky, since the bits
            // of the u128 probably won't line up with the bits of the u8's. as in:
            //
            //        ↓ start of row
            // [•  •  •  •  •  •  •  •][•  •  •  •  •  •  •  •][•  •  •  •  •
            //       [•  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •  •
            //
            // also, to which "side" of the u128 am i aligning the image data to?
            // probably the lowest numbers, so that the rightmost pixel of the row
            // falls onto the lowest, 2^0, bit.
            //
            // so... this may have gotten even more complex. should i consider
            // accessing the last bit of the row instead of the first?

            // idea: read in all the u8's as u128 (making sure to mask out the top bits
            // of the first one), bit-shift them appropriately, then 'or' them together

            // store last-pixel indices
            // note: bit_index_end will always be equivalent to bit_index_start - 1 (mod 8),
            // since 8 divides 128. idk whether to differentiate between them or not
            let (vec_index_end, bit_index_end) =
                xy_to_index(self.width - 1, y, self.width, self.height).unwrap();

            // first and last bytes need to be "masked" by bitshifting

            // last: remove bits of the next row (if bit index is 0 (lowest/rightmost bit), no shifting is needed)
            let last_byte = self.bits[vec_index_end] >> bit_index_end;

            // if the row fits in one byte, we are done
            // this requires self.width <= 8, but idk if that's worth checking for
            if vec_index_start == vec_index_end {
                last_byte as u128
            } else {
                // first: remove bits of the previous row (if bit index is 7 (highest/leftmost bit), no shifting is needed)
                // we basically smush the top bits up into nothingness and then lower the number back down
                let first_byte =
                    (self.bits[vec_index_start] << (7 - bit_index_start)) >> (7 - bit_index_start);

                // time to construct the output u128
                let mut output: u128 = 0;
                output |= first_byte as u128;

                // add the intermittent bytes, if there are any
                if vec_index_end - vec_index_start > 1 {
                    for i in (vec_index_start + 1)..=(vec_index_end - 1) {
                        output <<= 8;
                        output |= self.bits[i] as u128;
                    }
                }

                // add the last byte, shifted appropriately
                output <<= 8 - (bit_index_end);
                output |= last_byte as u128;

                output
            }
        } else {
            // error, do something good here
            panic!()
        }
    }

    // ... very tricky to optimize
    // lazy solution for now: use set_bit
    pub fn set_row(&mut self, y: usize, row: u128) {
        if y < self.height {
            for x in 0..(self.width - 1) {
                self.set_bit(x, y, ((row >> ((self.width - 1) - x)) % 2) == 1);
            }
        } else {
            println!(
                "out-of-bounds write (y={} w={} h={})",
                y, self.width, self.height
            );
        }
    }

    // very easy to implement, why not add it
    pub fn invert(&mut self) {
        // note that this doesn't leave inaccessible bits as 0, so you can't generally rely on that being true
        for i in 0..self.bits.len() {
            self.bits[i] ^= 0xff;
        }
    }

    pub fn dims(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn debug_bits(&self) -> Vec<u8> {
        self.bits.clone()
    }

    pub fn debug_indices(&self, x: usize, y: usize) -> Option<(usize, u8)> {
        xy_to_index(x, y, self.width, self.height)
    }
}

// implementing the ^= operator
impl BitXorAssign for Img {
    fn bitxor_assign(&mut self, rhs: Self) {
        if (self.width, self.height) != (rhs.width, rhs.height) {
            panic!();
        } else {
            for i in 0..self.bits.len() {
                self.bits[i] ^= rhs.bits[i];
            }
        }
    }
}

// this implementation packs all the image data into one continuous
// stream of bits, without gaps, so some bytes will have data from multiple
// rows in them if the width isn't a multiple of 8
fn xy_to_index(x: usize, y: usize, w: usize, h: usize) -> Option<(usize, u8)> {
    // converts x-y coordinates to the pixel's vector/bit indices:
    // Some(n, i) == bit i of vec[n]

    // return None when coords are out of bounds
    // using >= because w and h start from 1, not 0
    if x >= w || y >= h {
        return None;
    }

    // x and y point to this location in the "bitstream"
    // that is, x bits + y full rows of bits
    let bit = x + w * y;

    // vector index: quotient of 'bit'
    // bit index: remainder of 'bit'
    // this is fine since it's an uninterrupted stream of bits
    // pixel order is left - MSB, right - LSB
    // i don't know why the "7 - (bit mod 8)" calculation
    // works, but it does seem to work, so...
    Some(((bit / 8) as usize, (7 - (bit % 8)) as u8))
}

// this code below assumes data is saved in a way where the rows all start with
// a new byte, which leaves empty space in the last byte of every row
// if the width isn't a multiple of 8. it's messy and i decided to abandon it
/* fn xy_to_index_brk(x: u32, y: u32, w: u32, h: u32) -> Option<((u32, u8))> {
    // converts xy coordinates to the pixel's vector/bit indices:
    // Some(n, i) => bit i in vec[n]
    // returns None when coords are out of bounds
    if x > w || y > h {
        return None;
    }

    /*
       // rounding up width to compute index
       let w_round = w + ((8 - (w % 8)) % 8);
       /* {
           if w % 8 == 0 {
               w
           } else {
               w + 8 - (w % 8)
           }
       }; */

       let n = (w_round * y + x - x % 8) / 8;
    */
    return Some((n, (x % 8) as u8));
} */
