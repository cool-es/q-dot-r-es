pub mod continuous;
pub mod qr_standard;
pub mod rowaligned;

// workaround i got from That Website... feels unintentional
// but good enough for what i'm working with right now
use secret::BitmapDebug;

pub trait Bitmap: BitmapDebug {
    fn new(width: usize, height: usize) -> Self;
    fn dims(&self) -> (usize, usize);

    // returns "false" if out-of-bounds
    fn set_bit(&mut self, x: usize, y: usize, bit: bool) -> bool {
        if let Some((n, i)) = self.debug_xy_to_index(x, y) {
            if bit {
                // set a 1 (bitwise 'or' w/ 1)
                self.debug_bits_mut()[n] |= 1 << i;
            } else {
                //set a 0 (bitwise 'and' w/ 0)
                self.debug_bits_mut()[n] &= !(1 << i);
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
        if let Some((n, i)) = self.debug_xy_to_index(x, y) {
            Some(((self.debug_bits()[n] >> i) & 1) == 1)
        } else {
            None
        }
    }
   // very very inefficient and basic, but it works (in theory. i haven't tested it)
    fn get_row(&self, y: usize) -> Option<u128> {
        let (width, height) = self.dims();

        if y >= height {
            return None;
        }

        let mut output: u128 = 0;

        for i in 0..width {
            output += u128::from(self.get_bit(i, y)?) << i;
        }

        Some(output)
    }
}

mod secret {
    pub trait BitmapDebug {
        fn debug_bits(&self) -> &Vec<u8>;
        fn debug_bits_mut(&mut self) -> &mut Vec<u8>;
        fn debug_xy_to_index(&self, x: usize, y: usize) -> Option<(usize, u8)>;
        fn debug_index_to_xy(&self, vec_index: usize, bit_index: u8) -> Option<(usize, usize)>;
    }
}
