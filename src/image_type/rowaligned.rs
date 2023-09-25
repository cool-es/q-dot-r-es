// image format with gaps in its byte data: start of rows are byte aligned
pub struct ImgRowAligned {
    width: usize,
    height: usize,
    bits: Vec<u8>,
}

impl ImgRowAligned {
        // functions to input/output XBM data - may be complicated due to different byte format
    // &str for sake of being used with constant strings
    pub fn from_xbm(input: &str) -> Self {
        todo!()
    }
    pub fn as_xbm(&self) -> String {
        todo!()
    }
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
