//! The info-holding structure.

// NOTES:
// it seems like WASM has access to 17 * 65536 = 1114112 bytes, or 1.0625 MiB
// this works out to about 35 of my (enormous) bitmap arrays + 17 KiB

use super::{Byte, NativeInt};

// bitmap array of 177^2 bytes, the size of the largest QR version
// const MAX_SIZE: usize = 31329;
// bitmap array of a version 15 code
const MAX_SIZE: usize = 5929;
pub type BmpArray = [Byte; MAX_SIZE];
pub const BLANK_BMP: BmpArray = [0; MAX_SIZE];

// coordinates, for tracking the snaking pattern of data
pub type BmpCoords = [[NativeInt; 2]; MAX_SIZE];
pub static BLANK_COORDS: BmpCoords = [[0; 2]; MAX_SIZE];

// the structure holding information about the qr code
#[derive(Debug, Clone)]
pub struct Info {
    // bitmap without a mask
    pub bitmap_fcode: BmpArray,

    // bitmap without a mask
    pub bitmap_nomask: BmpArray,

    // bitmap with a mask (readable qr code)
    pub bitmap: BmpArray,

    // only the binary "badstream"
    pub bitmap_badstream: BmpArray,

    // only the base QR alignment patterns etc
    pub bitmap_base: BmpArray,

    // all of the masks applied to otherwise blank bitmaps
    pub bitmap_masks: [BmpArray; 8],

    // the width/height of the qr code
    pub dims: NativeInt,

    pub bitstream: [Byte; MAX_SIZE],

    // the coordinates for the bitstream
    pub bitstream_coords: [(NativeInt, NativeInt); MAX_SIZE],

    // the small rectangle in larger qr codes
    pub ecblock_data: [Byte; MAX_SIZE],

    // the mask data positioned around the alignment patterns
    pub format_info: [Byte; 2],

    // which mask was chosen
    pub mask: NativeInt,

    // mode/byte data, as chosen by find_best_mode_optimization()
    // reprocessed into a format more readable by javascript
    pub modes: [Byte; MAX_SIZE],

    // which version was chosen
    pub version: NativeInt,

    pub penalties: [u32; 4],
}

pub const BLANK_INFO: Info = Info {
    bitmap: BLANK_BMP,
    bitmap_badstream: BLANK_BMP,
    bitmap_base: BLANK_BMP,
    bitmap_fcode: BLANK_BMP,
    bitmap_masks: [BLANK_BMP; 8],
    bitmap_nomask: BLANK_BMP,

    bitstream_coords: [(0, 0); MAX_SIZE],
    bitstream: [0; MAX_SIZE],
    ecblock_data: [0; MAX_SIZE],
    modes: [Byte::MAX; MAX_SIZE],

    format_info: [0; 2],
    penalties: [0; 4],

    dims: NativeInt::MAX,
    mask: NativeInt::MAX,
    version: NativeInt::MAX,
};

impl Info {
    pub const fn new() -> Info {
        BLANK_INFO
    }

    pub fn clear(&mut self) {
        *self = BLANK_INFO;
    }
}
