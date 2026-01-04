//! The info-holding structure.

use super::{Byte, NativeInt};

// the structure holding information about the qr code
#[derive(Debug, Clone)]
pub(crate) struct Info {
    // bitmap without a mask
    pub(crate) bitmap_nomask: Vec<Byte>,

    // bitmap with a mask (readable qr code)
    pub(crate) bitmap: Vec<Byte>,

    // TBD
    pub(crate) codewords: Vec<Byte>,

    // the small rectangle in larger qr codes
    pub(crate) ecblock_data: Vec<Byte>,

    // the mask data positioned around the alignment patterns
    pub(crate) corner_mask_data: Vec<Byte>,

    // which mask was chosen
    pub(crate) mask: NativeInt,

    // mode/byte data, as chosen by find_best_mode_optimization()
    // reprocessed into a format more readable by wasm
    pub(crate) modes: Vec<Byte>,

    // which version was chosen
    pub(crate) version: NativeInt,
}

impl Info {
    pub(crate) const fn new() -> Info {
        Info {
            bitmap_nomask: Vec::new(),
            bitmap: Vec::new(),
            codewords: Vec::new(),
            ecblock_data: Vec::new(),
            corner_mask_data: Vec::new(),
            mask: NativeInt::MAX,
            modes: Vec::new(),
            version: NativeInt::MAX,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.bitmap_nomask.clear();
        self.bitmap.clear();
        self.codewords.clear();
        self.corner_mask_data.clear();
        self.ecblock_data.clear();
        self.mask = NativeInt::MAX;
        self.modes.clear();
        self.version = NativeInt::MAX;
    }
}
