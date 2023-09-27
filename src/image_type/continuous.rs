use super::{Bitmap, BitmapDebug};
use std::ops::BitXorAssign;
// 2-dimensional one bit data type, stored in a vector of u8's
pub struct Img {
    // width, height fields are private so that they can't be mutated
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) bits: Vec<u8>,
}

// general format-specific methods
impl Img {
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

    // "stamp" function, to copy a smaller bitmap onto a bigger one
    // (e.g. a qr alignment square onto a qr code). the x/y coords
    // are aligned to the stamp's top left corner
    pub fn rubberstamp(&mut self, stamp: &Self, x: usize, y: usize) {
        todo!()
    }

    pub fn make_rowaligned(self) -> super::rowaligned::ImgRowAligned {
        let (width, height) = self.dims();
        if self.width % 8 == 0 {
            // nothing needs to be done
            // note that the fields match up but the types don't!
            return super::rowaligned::ImgRowAligned {
                width,
                height,
                bits: self.bits,
            };
        }

        //  really awful implementation here, but,
        let mut output = super::rowaligned::ImgRowAligned::new(width, height);

        for y in 0..self.height {
            let row = self.get_row(y).unwrap();
            for x in 0..self.width {
                output.set_bit(x, y, (row >> (width - (x + 1))) & 1 == 1);
            }
        }

        output
    }
}

impl Bitmap for Img {
    fn new(width: usize, height: usize) -> Self {
        let mut bits: Vec<u8> = Vec::new();

        // resize vector to contain the maximum needed amount of
        // elements – that is, the index of the byte containing the last pixel
        // risk of fencepost error: resize() counts from 1, xy_to_index from 0
        bits.resize(
            xy_to_index(width - 1, height - 1, width, height).unwrap().0 + 1,
            0,
        );

        Img {
            width: width,
            height: height,
            bits,
        }
    }

    fn dims(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    // returns "false" if out-of-bounds
    fn set_bit(&mut self, x: usize, y: usize, bit: bool) -> bool {
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

    fn get_bit(&self, x: usize, y: usize) -> Option<bool> {
        let ref this = self;
        if let Some((n, i)) = xy_to_index(x, y, this.width, this.height) {
            Some(((this.bits[n] >> i) & 1) == 1)
        } else {
            None
        }
    }

    //returns a given row as the bits of a u128
    fn get_row(&self, y: usize) -> Option<u128> {
        if self.width > 128 {
            // won't fit
            return None;
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
                Some(last_byte as u128)
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

                Some(output)
            }
        } else {
            // error, do something good here
            return None;
        }
    }
}

impl BitmapDebug for Img {
    fn debug_bits(&self) -> &Vec<u8> {
        &self.bits
    }
    fn debug_bits_mut(&mut self) -> &mut Vec<u8> {
        &mut self.bits
    }
    fn debug_xy_to_index(&self, x: usize, y: usize) -> Option<(usize, u8)> {
        xy_to_index(x, y, self.width, self.height)
    }
    fn debug_index_to_xy(&self, vec_index: usize, bit_index: u8) -> Option<(usize, usize)> {
        index_to_xy(vec_index, bit_index, self.width, self.height)
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

// implementing clone()
impl Clone for Img {
    // returns a carbon copy of the original, including inaccessible bits
    fn clone(&self) -> Self {
        Img {
            width: self.width,
            height: self.height,
            bits: self.bits.clone(),
        }
    }
}

impl From<super::rowaligned::ImgRowAligned> for Img {
    fn from(value: super::rowaligned::ImgRowAligned) -> Self {
        value.make_continuous()
    }
}
// this implementation packs all the image data into one continuous
// stream of bits, without gaps, so some bytes will have data from multiple
// rows in them if the width isn't a multiple of 8
pub(super) fn xy_to_index(x: usize, y: usize, w: usize, h: usize) -> Option<(usize, u8)> {
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

pub(super) fn index_to_xy(
    vec_index: usize,
    bit_index: u8,
    w: usize,
    h: usize,
) -> Option<(usize, usize)> {
    let bit = (vec_index * 8) + (7 - bit_index) as usize;
    let x = bit % w;
    let y = (bit - x) / w;
    if y >= h {
        None
    } else {
        Some((x, y))
    }
}
