//! The info-holding structure.

use super::{Byte, NativeInt};

// the structure holding information about the qr code
#[derive(Debug, Clone)]
pub(crate) struct Info {
    // bitmap without a mask
    pub bitmap_fcode: Vec<Byte>,

    // bitmap without a mask
    pub bitmap_nomask: Vec<Byte>,

    // bitmap with a mask (readable qr code)
    pub(crate) bitmap: Vec<Byte>,

    // TBD
    pub(crate) codewords: Vec<Byte>,

    // the small rectangle in larger qr codes
    pub(crate) ecblock_data: Vec<Byte>,

    // the mask data positioned around the alignment patterns
    pub format_info: [Byte; 2],

    // which mask was chosen
    pub(crate) mask: NativeInt,

    // mode/byte data, as chosen by find_best_mode_optimization()
    // reprocessed into a format more readable by javascript
    pub(crate) modes: Vec<Byte>,

    // which version was chosen
    pub(crate) version: NativeInt,
}

impl Info {
    pub(crate) const fn new() -> Info {
        Info {
            bitmap_fcode: Vec::new(),
            bitmap_nomask: Vec::new(),
            bitmap: Vec::new(),
            codewords: Vec::new(),
            ecblock_data: Vec::new(),
            format_info: [0; 2],
            mask: NativeInt::MAX,
            modes: Vec::new(),
            version: NativeInt::MAX,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.bitmap_fcode.clear();
        self.bitmap_nomask.clear();
        self.bitmap.clear();
        self.codewords.clear();
        self.ecblock_data.clear();
        self.format_info = [0; 2];
        self.mask = NativeInt::MAX;
        self.modes.clear();
        self.version = NativeInt::MAX;
    }
}
