//! The info-holding structure.

use super::{Byte, NativeInt};

// bitmap array of 177^2 bytes, the size of the largest QR version
const MAX_SIZE: usize = 31329;
pub type BmpArray = [Byte; MAX_SIZE];
pub const BLANK_BMP: BmpArray = [0; MAX_SIZE];

// the structure holding information about the qr code
#[derive(Debug, Clone)]
pub struct Info {
    // bitmap without a mask
    pub bitmap_fcode: BmpArray,

    // bitmap without a mask
    pub bitmap_nomask: BmpArray,

    // bitmap with a mask (readable qr code)
    pub bitmap: BmpArray,

    // TBD
    pub codewords: Vec<Byte>,

    // the small rectangle in larger qr codes
    pub ecblock_data: Vec<Byte>,

    // the mask data positioned around the alignment patterns
    pub format_info: [Byte; 2],

    // which mask was chosen
    pub mask: NativeInt,

    // mode/byte data, as chosen by find_best_mode_optimization()
    // reprocessed into a format more readable by javascript
    pub modes: Vec<Byte>,

    // which version was chosen
    pub version: NativeInt,
}

impl Info {
    pub const fn new() -> Info {
        Info {
            bitmap_fcode: BLANK_BMP,
            bitmap_nomask: BLANK_BMP,
            bitmap: BLANK_BMP,
            codewords: Vec::new(),
            ecblock_data: Vec::new(),
            format_info: [0; 2],
            mask: NativeInt::MAX,
            modes: Vec::new(),
            version: NativeInt::MAX,
        }
    }

    pub fn clear(&mut self) {
        self.bitmap_fcode = BLANK_BMP;
        self.bitmap_nomask = BLANK_BMP;
        self.bitmap = BLANK_BMP;
        self.codewords.clear();
        self.ecblock_data.clear();
        self.format_info = [0; 2];
        self.mask = NativeInt::MAX;
        self.modes.clear();
        self.version = NativeInt::MAX;
    }
}
