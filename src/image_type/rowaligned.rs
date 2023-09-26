// image format with gaps in its byte data: start of rows are byte aligned
pub struct ImgRowAligned {
    pub width: usize,
    pub height: usize,
    bits: Vec<u8>,
}

impl ImgRowAligned {
    // copied wholesale from Img
    pub fn new(w: usize, h: usize) -> Self {
        let mut vec: Vec<u8> = Vec::new();

        // resize vector to contain the maximum needed amount of
        // elements – that is, the index of the byte containing the last pixel
        // risk of fencepost error: resize() counts from 1, xy_to_index from 0
        vec.resize(xy_to_index(w - 1, h - 1, w, h).unwrap().0 + 1, 0);

        ImgRowAligned {
            width: w,
            height: h,
            bits: vec,
        }
    }

    // copied wholesale from Img
    // very easy to implement, why not add it
    pub fn invert(&mut self) {
        // note that this doesn't leave inaccessible bits as 0, so you can't generally rely on that being true
        for i in 0..self.bits.len() {
            self.bits[i] ^= 0xff;
        }
    }

    // copied wholesale from Img
    pub fn dims(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    // copied wholesale from Img
    pub fn debug_bits(&self) -> Vec<u8> {
        self.bits.clone()
    }

    // copied wholesale from Img
    pub fn debug_indices(&self, x: usize, y: usize) -> Option<(usize, u8)> {
        xy_to_index(x, y, self.width, self.height)
    }

    // functions to input/output XBM data
    // &str for sake of being used with constant strings
    /*
    pub fn from_xbm(input: &str) -> Option<Self> {
            // note that XBM uses reverse byte order (leftmost pixel is the 2^0 bit)

            //split at the start of the byte data
            let (dims, bytes) = input.split_once('{')?;

            let mut dimensions = dims.lines().map(|x| {
                if x.starts_with('#') {
                    x.trim()
                        .split_whitespace()
                        .rev()
                        .next()?
                        .parse::<usize>()
                        .ok()
                } else {
                    None
                }
            });
            // .map(|x| Some(x.split_whitespace().rev().next()?.parse::<usize>().ok()?));

            let width = dimensions.next()??;
            let height = dimensions.next()??;

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
                    bits.push(byte.trim().parse::<u8>().ok()?.reverse_bits());
                }
            }

            Some(ImgRowAligned {
                width: width,
                height: height,
                bits: bits,
            })
        }
     */
    // same as previous function but with (incomplete) error handling
    pub fn from_xbm(input: &str) -> Result<Self, &str> {
        // note that XBM uses reverse byte order (leftmost pixel is the 2^0 bit)

        //split at the start of the byte data
        let (dims, bytes) = input.split_once('{').ok_or("could not split at {")?;

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
                bits.push(byte.trim().parse::<u8>().map_err(|_| "wuh")?.reverse_bits());
            }
        }

        Ok(ImgRowAligned {
            width: width,
            height: height,
            bits: bits,
        })
    }

    pub fn as_xbm(&self, name: &str) -> String {
        let mut output = format!(
            "#define {}_width {}\n#define {}_height {}\n",
            name, self.width, name, self.height
        );
        output.push_str("static unsigned char test_bits[] = {");
        for n in 0..self.bits.len() {
            if n % 12 == 0 {
                output.push_str("\n    ");
            }
            output.push_str(format!("[{:#02}]", self.bits[n].reverse_bits()).as_str());

            if n < self.bits.len() - 1 {
                output.push_str(", ");
            } else {
                output.push_str("};");
            }
        }
        output
    }
}

// this code below assumes data is saved in a way where the rows all start with
// a new byte, which leaves empty space in the last byte of every row
// if the width isn't a multiple of 8
pub(super) fn xy_to_index(x: usize, y: usize, w: usize, h: usize) -> Option<((usize, u8))> {
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
    todo!()
}

const XBM_EXAMPLE: &str = "#define test_width 16
#define test_height 16
static unsigned char test_bits[] = {
   0x01, 0x00, 0x02, 0x00, 0x04, 0x00, 0x08, 0x00, 0x10, 0x00, 0x20, 0x00,
   0x40, 0x00, 0x80, 0x00, 0xfe, 0xff, 0xfd, 0xff, 0xfb, 0xff, 0xf7, 0xff,
   0xef, 0xff, 0xdf, 0xff, 0xbf, 0xff, 0x7f, 0xff };
";
