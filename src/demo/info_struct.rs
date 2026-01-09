//! The info-holding structure.

use super::{Byte, NativeInt};

// bitmap array of 177^2 bytes, the size of the largest QR version
const MAX_SIZE: usize = 31329;
pub type BmpArray = [Byte; MAX_SIZE];
pub const BLANK_BMP: BmpArray = [0; MAX_SIZE];

// coordinates, for tracking the snaking pattern of data
pub type BmpCoords = [[NativeInt; 2]; MAX_SIZE];
pub const BLANK_COORDS: BmpCoords = [[0; 2]; MAX_SIZE];

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

    pub bitstream: Vec<Byte>,

    // the coordinates for the bitstream
    pub bitstream_coords: Vec<(NativeInt, NativeInt)>,

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

    pub penalties: [u32; 4],
}

impl Info {
    pub const fn new() -> Info {
        Info {
            bitmap: BLANK_BMP,
            bitmap_badstream: BLANK_BMP,
            bitmap_base: BLANK_BMP,
            bitmap_fcode: BLANK_BMP,
            bitmap_masks: [BLANK_BMP; 8],
            bitmap_nomask: BLANK_BMP,

            bitstream_coords: Vec::new(),
            bitstream: Vec::new(),
            ecblock_data: Vec::new(),
            modes: Vec::new(),

            format_info: [0; 2],

            dims: NativeInt::MAX,
            mask: NativeInt::MAX,
            version: NativeInt::MAX,
            penalties: [0; 4],
        }
    }

    pub fn clear(&mut self) {
        let Info {
            bitmap_badstream,
            bitmap_base,
            bitmap_fcode,
            bitmap_masks,
            bitmap_nomask,
            bitmap,
            bitstream_coords,
            bitstream,
            dims,
            ecblock_data,
            format_info,
            mask,
            modes,
            penalties,
            version,
        } = self;

        *bitmap = BLANK_BMP;
        *bitmap_badstream = BLANK_BMP;
        *bitmap_base = BLANK_BMP;
        *bitmap_fcode = BLANK_BMP;
        *bitmap_masks = [BLANK_BMP; 8];
        *bitmap_nomask = BLANK_BMP;

        bitstream.clear();
        bitstream_coords.clear();
        ecblock_data.clear();
        modes.clear();

        *format_info = [0; 2];
        *dims = NativeInt::MAX;
        *mask = NativeInt::MAX;
        *version = NativeInt::MAX;
    }
}
