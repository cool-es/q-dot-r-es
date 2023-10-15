use super::{Bitmap, BitmapDebug};

// image format with gaps in its byte data: start of rows are byte aligned
pub struct ImgRowAligned {
    // width, height fields are private so that they can't be mutated
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) bits: Vec<u8>,
}

impl ImgRowAligned {
    pub fn mask_set(&mut self, pattern: &ImgRowAligned, mask: &ImgRowAligned) {
        if self.dims() != pattern.dims() || self.dims() != mask.dims() {
            // size mismatch
            panic!()
        }

        for i in 0..self.bits.len() {
            // (P & M) | (S & !M)
            // if M is 1, output is == P
            // if M is 0, output is == S

            // changed - had the opposite behavior from what was intended
            self.bits[i] = (pattern.bits[i] & !mask.bits[i]) | (self.bits[i] & mask.bits[i]);
        }
    }

    pub fn invert(&mut self) {
        // note that this doesn't leave inaccessible bits as 0, so you can't generally rely on that being true
        for i in 0..self.bits.len() {
            self.bits[i] ^= 0xff;
        }
    }

    pub fn make_continuous(self) -> super::continuous::Img {
        let (width, height) = self.dims();
        if self.width % 8 == 0 {
            // nothing needs to be done
            // note that the fields match up but the types don't!
            return super::continuous::Img {
                width,
                height,
                bits: self.bits,
            };
        }

        //  really awful implementation here, but,
        let mut output = super::continuous::Img::new(width, height);

        for y in 0..self.height {
            let row = self.get_row(y).unwrap();
            for x in 0..self.width {
                output.set_bit(x, y, (row >> (width - (x + 1))) & 1 == 1);
            }
        }

        output
    }

    // same as previous function but with (incomplete) error handling
    // the code here isn't great but it's passable
    pub fn from_xbm(input: &str) -> Result<Self, &str> {
        // note that XBM uses reverse byte order (leftmost pixel is the 2^0 bit)

        //split at the start of the byte data
        let (dims, bytes) = input.split_once('{').ok_or("could not split at {{")?;

        let mut dimensions = dims.lines().map(|x| {
            if x.starts_with("#define") {
                x.trim()
                    .split_whitespace()
                    .rev()
                    .next()
                    .ok_or("something messed up??")?
                    .parse::<usize>()
                    .map_err(|_| "parse error")
            } else {
                Err("dimension lines not starting with #define")
            }
        });
        // .map(|x| Some(x.split_whitespace().rev().next()?.parse::<usize>().ok()?));

        let width = dimensions.next().ok_or("width failed to parse")??;
        let height = dimensions.next().ok_or("height failed to parse")??;

        let mut bits = Vec::new();

        // remove whitespace, remove final bracket (returns None if unsuccessful),
        // unwrap or use the aforementioned value, split on commas
        for byte in bytes
            .trim()
            // .strip_suffix("};")
            // .unwrap_or(bytes.trim())
            // .split(", ")
            .split(|x| !char::is_alphanumeric(x))
        {
            if byte.len() != 0 {
                bits.push(
                    {
                        // debugging code, prints offending characters
                        let z = u8::from_str_radix(byte.trim().split_once('x').unwrap().1, 16);
                        if z.is_err() {
                            println!("{}", byte);
                        }
                        z
                    }
                    .map_err(|_| "wuh")?
                    .reverse_bits(),
                );
            }
        }

        /* bytes
        .trim()
        .split(|x| !char::is_alphanumeric(x))
        .map(|byte| {
            if byte.len() != 0 {
                (bits.push(byte.trim().parse::<u8>().map_err(|_| "wuh")?.reverse_bits()));
            }
        }); */

        Ok(ImgRowAligned {
            width,
            height,
            bits,
        })
    }

    // the curly brackets here really mess with my syntax highlighting... but the code itself is correct
    pub fn as_xbm(&self, name: &str) -> String {
        let mut output = format!(
            "#define {}_width {}\n#define {}_height {}\n",
            name, self.width, name, self.height
        );
        output.push_str(format!("static unsigned char {}_bits[] = {{", name).as_str());
        for n in 0..self.bits.len() {
            if n % 12 == 0 {
                output.push_str("\n  ");
            }
            output.push_str(format!(" 0x{:02x}", self.bits[n].reverse_bits()).as_str());

            if n < self.bits.len() - 1 {
                output.push_str(",");
            } else {
                output.push_str(" };\n");
            }
        }
        // output.push_str(format!("// run with:\n// cargo r -q > {}.xbm\n",name).as_str());
        output
    }

    pub fn from_xbm_debug() -> Self {
        Self::from_xbm(XBM_EXAMPLE).unwrap()
    }
}

impl From<super::continuous::Img> for ImgRowAligned {
    fn from(value: super::continuous::Img) -> Self {
        value.make_rowaligned()
    }
}

// this code below assumes data is saved in a way where the rows all start with
// a new byte, which leaves empty space in the last byte of every row
// if the width isn't a multiple of 8
pub(super) fn xy_to_index(x: usize, y: usize, w: usize, h: usize) -> Option<(usize, u8)> {
    // converts xy coordinates to the pixel's vector/bit indices:
    // Some(n, i) => bit i in vec[n]
    // returns None when coords are out of bounds
    if x > w || y > h {
        return None;
    }

    let row_bytes = if w % 8 == 0 { w / 8 } else { w / 8 + 1 };

    let n = y * row_bytes + x / 8;

    // highest bit index is to the left
    let i = (7 - (x % 8)) as u8;

    Some((n, i))
}

pub(super) fn index_to_xy(
    vec_index: usize,
    bit_index: u8,
    w: usize,
    h: usize,
) -> Option<(usize, usize)> {
    let row_bytes = if w % 8 == 0 { w / 8 } else { w / 8 + 1 };

    let x = (vec_index % row_bytes) * 8 + (7 - bit_index) as usize;
    let y = vec_index / row_bytes;
    if x >= w || y >= h {
        return None;
    }
    Some((x, y))
}

const XBM_EXAMPLE: &str = "#define test_width 16
#define test_height 16
static unsigned char test_bits[] = {
   0x01, 0x00, 0x02, 0x00, 0x04, 0x00, 0x08, 0x00, 0x10, 0x00, 0x20, 0x00,
   0x40, 0x00, 0x80, 0x00, 0xfe, 0xff, 0xfd, 0xff, 0xfb, 0xff, 0xf7, 0xff,
   0xef, 0xff, 0xdf, 0xff, 0xbf, 0xff, 0x7f, 0xff };
";

// implementing clone()
impl Clone for ImgRowAligned {
    // returns a carbon copy of the original, including inaccessible bits
    fn clone(&self) -> Self {
        ImgRowAligned {
            width: self.width,
            height: self.height,
            bits: self.bits.clone(),
        }
    }
}

impl Bitmap for ImgRowAligned {
    fn new(width: usize, height: usize) -> Self {
        let mut bits: Vec<u8> = Vec::new();

        // resize vector to contain the maximum needed amount of
        // elements – that is, the index of the byte containing the last pixel
        // risk of fencepost error: resize() counts from 1, xy_to_index from 0
        bits.resize(
            xy_to_index(width - 1, height - 1, width, height).unwrap().0 + 1,
            0,
        );

        ImgRowAligned {
            width: width,
            height: height,
            bits,
        }
    }

    fn dims(&self) -> (usize, usize) {
        (self.width, self.height)
    }

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
        if let Some((n, i)) = xy_to_index(x, y, self.width, self.height) {
            Some(((self.bits[n] >> i) & 1) == 1)
        } else {
            None
        }
    }

    fn get_row(&self, y: usize) -> Option<u128> {
        if y >= self.height {
            return None;
        }
        let mut output: u128 = 0;

        for i in self.debug_xy_to_index(0, y)?.0..=self.debug_xy_to_index(self.width - 1, y)?.0 {
            output <<= 8;
            output += self.bits[i] as u128;
        }

        output >>= self.debug_xy_to_index(self.width - 1, y)?.1 as u128;

        Some(output)
    }
}

impl BitmapDebug for ImgRowAligned {
    fn debug_bits(&self) -> &Vec<u8> {
        &self.bits
    }

    fn debug_bits_mut(&mut self) -> &mut Vec<u8> {
        &mut self.bits
    }

    fn debug_index_to_xy(&self, vec_index: usize, bit_index: u8) -> Option<(usize, usize)> {
        index_to_xy(vec_index, bit_index, self.width, self.height)
    }

    fn debug_xy_to_index(&self, x: usize, y: usize) -> Option<(usize, u8)> {
        xy_to_index(x, y, self.width, self.height)
    }
}
