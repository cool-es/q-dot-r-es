pub mod continuous;
pub mod qr_standard;
pub mod rowaligned;

pub(crate) trait Bitmap {
    fn new(width: usize, height: usize) -> Self;
    fn dims(&self) -> (usize, usize);
    fn get_bit(&self, x: usize, y: usize) -> Option<bool>;
    fn set_bit(&mut self, x: usize, y: usize, bit: bool) -> bool;

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

// impl<T> Clone for T
// where
//     T: Bitmap,
// {
//     fn clone(&self) -> Self {
//         todo!()
//     }
// }
