/// image format with gaps in its byte data: start of rows are byte aligned
#[derive(Clone)]
pub struct Bitmap {
    // width, height fields are private so that they can't be mutated
    width: usize,
    height: usize,
    bits: Vec<u8>,

    // whether the qr code has a designated "quiet zone"
    border: bool,
}

impl Bitmap {
    // rescaling with naive nearest-neighbor implementation
    pub fn scale(self, target_width: Option<usize>) -> Bitmap {
        // define this code snippet as a function due to re-use
        fn bitmap_subfunction(input: Bitmap, target_width: usize) -> Bitmap {
            let mut output = Bitmap::new(target_width, target_width);
            let factor = input.width as f32 / target_width as f32;
            for i in 0..target_width {
                // horizontal step
                let fi = (i as f32 * factor).trunc() as usize;
                if input.border && (fi < 8 || (input.width - fi) < 8) {
                    continue;
                }
                for j in 0..target_width {
                    // vertical step
                    let fj = (j as f32 * factor).trunc() as usize;
                    if input.border && (fj < 8 || (input.height - fj) < 8) {
                        continue;
                    }

                    if input.get_bit(fi, fj).expect("scaling") {
                        output.set_bit(i, j, true);
                    }
                }
            }
            output
        }

        // start parsing input
        if let Some(target_width) = target_width {
            // check if input is ok
            if target_width == 0 {
                // no scaling to be applied
                return self;
            } else if target_width < self.width {
                // downscaling is not allowed!
                eprintln!("Not rescaled: requested width is smaller than the QR code");
                return self;
            } else if target_width > 5000 {
                // too big
                eprintln!("Not rescaled: requested width is too large (>5000 px)");
                return self;
            }

            // input is fine, rescale
            bitmap_subfunction(self, target_width)
        } else {
            // no input; apply default scaling (512 px)
            bitmap_subfunction(self, 512)
        }
    }

    // add 8-pixel "quiet zone" border to bitmap. not fast (yet), but sensible
    pub fn add_border(self) -> Bitmap {
        if self.border {
            return self;
        }
        let mut output = Bitmap::new(self.width + 16, self.height + 16);
        for i in 0..self.width {
            for j in 0..self.height {
                if self.get_bit(i, j).expect("border") {
                    output.set_bit(i + 8, j + 8, true);
                }
            }
        }
        output.border = true;
        output
    }

    /// Exports the bitmap as an uncompressed black-and-white BMP file.
    pub fn as_bmp(&self) -> Vec<u8> {
        // row bytes in final pixel array
        // necessary for padding!
        let bmp_byte_width = self.width.div_ceil(32) * 4;

        // number of pixel-data bytes
        let imagesize = self.height * bmp_byte_width;
        // image size + data offset
        let filesize = imagesize + 0x3e;

        const BMP_TEMPLATE: [u8; 62] = [
            0x42, 0x4d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3e, 0x00, 0x00, 0x00,
            0x28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x13, 0x0b, 0x00, 0x00,
            0x13, 0x0b, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0xff, 0xff,
            0xff, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let mut output = Vec::with_capacity(filesize);
        output.extend(BMP_TEMPLATE);

        // fill in data
        for [data, byte_offset] in [
            [filesize, 0x02],
            [self.width, 0x12],
            [self.height, 0x16],
            [imagesize, 0x22],
        ] {
            output[byte_offset..byte_offset + 4].copy_from_slice(&(data as u32).to_le_bytes());
        }

        // fill image data space with zero bytes
        let mut bmp_data = vec![0x00; imagesize];

        // padding procedure; move data from internal format
        let i_f_data = &self.bits;
        let i_f_byte_width = i_f_data.len() / self.height;

        // move internal-format data into bmp. bmp stores lines
        // bottom to top, so the internal-format data is reversed
        for (bmp_row, i_f_row) in bmp_data
            .chunks_exact_mut(bmp_byte_width)
            .zip(i_f_data.chunks_exact(i_f_byte_width).rev())
        {
            // copy data into the beginning of every row
            bmp_row[..i_f_byte_width].copy_from_slice(i_f_row);
        }

        output.extend(&bmp_data);
        output
    }

    /// `as_xbm()`, but with an added 8 pixel quiet-zone border on all sides
    pub fn as_xbm(&self, name: &str) -> String {
        assert!(
            name.is_ascii() && !name.contains(char::is_whitespace),
            "name must be ascii and cannot contain whitespace"
        );
        // handle data separately
        let mut data = String::new();
        for n in 0..self.bits.len() {
            data.push_str(format!("0x{:02x},", self.bits[n].reverse_bits()).as_str());
        }
        // remove the last comma
        data.pop();

        // divide into 12 columns
        let mut nicedata = String::new();
        for string_chunk in Vec::from_iter(data.split(',')).chunks(12) {
            nicedata.push_str("    ");
            for byte in string_chunk.iter() {
                nicedata.push_str(byte);
                nicedata.push_str(", ")
            }
            nicedata.pop();
            nicedata.push('\n');
        }

        format!(
            "#define {}_width {}\n#define {}_height {}\nstatic unsigned char {}_bits[] = {{\n{}}};\n",
            name, self.width, name, self.height, name, nicedata,
        )
    }

    pub fn new(width: usize, height: usize) -> Self {
        let bits: Vec<u8> =
            vec![0; xy_to_index(width - 1, height - 1, width, height).unwrap().0 + 1];

        // resize vector to contain the maximum needed amount of
        // elements – that is, the index of the byte containing the last pixel
        // risk of fencepost error: resize() counts from 1, xy_to_index from 0

        Bitmap {
            width,
            height,
            bits,
            border: false,
        }
    }

    pub fn dims(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn set_bit(&mut self, x: usize, y: usize, bit: bool) -> bool {
        match xy_to_index(x, y, self.width, self.height) {
            Some((n, i)) => {
                if bit {
                    // set a 1 (bitwise 'or' w/ 1)
                    self.bits[n] |= 1 << i;
                } else {
                    //set a 0 (bitwise 'and' w/ 0)
                    self.bits[n] &= !(1 << i);
                }
                true
            }
            None => false,
        }
    }

    pub fn get_bit(&self, x: usize, y: usize) -> Option<bool> {
        xy_to_index(x, y, self.width, self.height).map(|(n, i)| ((self.bits[n] >> i) & 1) == 1)
    }

    pub fn debug_bits(&self) -> &Vec<u8> {
        &self.bits
    }

    pub fn debug_bits_mut(&mut self) -> &mut Vec<u8> {
        &mut self.bits
    }

    pub fn debug_index_to_xy(&self, vec_index: usize, bit_index: u8) -> Option<(usize, usize)> {
        index_to_xy(vec_index, bit_index, self.width, self.height)
    }

    // // // // // // // // // // // // // // // // // //
    /// set an already initialized bitmap to be identical to a different one
    /// only for use with wasm
    #[cfg(feature = "demo")]
    pub fn set_to(&mut self, x: &Bitmap) {
        self.bits.clear();
        self.bits.extend(x.bits.iter());
        self.width = x.width;
        self.height = x.height;
        self.border = x.border;
    }
    // // // // // // // // // // // // // // // // // //

    // // // // // // // // // // // // // // // // // //
    /// Create a new bitmap with the same dimensions as a different one.
    #[cfg(feature = "demo")]
    pub fn blank_clone(to_copy: &Self) -> Self {
        Self::new(to_copy.width, to_copy.height)
    }
    // // // // // // // // // // // // // // // // // //
}

/// converts xy coordinates to the pixel's vector/bit indices:
/// `(n, i)` = bit `i` of `vec[n]`.
/// returns None when coords are out of bounds.
///
/// assumes data is saved in a way where the rows all start with
/// a new byte, which leaves empty space in the last byte of every row
/// if the width isn't a multiple of 8.
pub fn xy_to_index(x: usize, y: usize, w: usize, h: usize) -> Option<(usize, u8)> {
    if x >= w || y >= h {
        return None;
    }

    let row_bytes = w.div_ceil(8);
    let n = y * row_bytes + x / 8;

    // highest bit index is to the left
    let i = (7 - (x % 8)) as u8;

    Some((n, i))
}

pub fn index_to_xy(vec_index: usize, bit_index: u8, w: usize, h: usize) -> Option<(usize, usize)> {
    let row_bytes = w.div_ceil(8);

    let x = (vec_index % row_bytes) * 8 + (7 - bit_index) as usize;
    let y = vec_index / row_bytes;
    if x >= w || y >= h {
        return None;
    }
    Some((x, y))
}
