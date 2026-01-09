//! operations to be called by the end user

use {
    crate::{
        demo::{info, Byte, NativeInt},
        image,
        qr_standard::bitstream::{Mode, Token},
    },
    std::iter::repeat,
};

// the specific static variable storing the info at a specific place in memory
static mut INFO_STATE: info::Info = info::Info::new();

// the unsafe "swiss army knife function"
#[allow(static_mut_refs)]
pub(crate) fn process_info<F, K>(f: F) -> K
where
    F: FnOnce(&mut info::Info) -> K,
{
    unsafe { f(&mut INFO_STATE) }
}

// returns pointer to and byte length of a vector
pub(crate) fn ptr_and_len<T>(v: &'static T) -> (NativeInt, NativeInt)
where
    Vec<Byte>: From<&'static T>,
{
    let bytes: Vec<Byte> = v.into();
    (bytes.as_ptr() as NativeInt, bytes.len() as NativeInt)
}

pub fn mask(mask: Option<NativeInt>) -> NativeInt {
    if let Some(mask) = mask {
        // set new value
        process_info(|x| {
            let old = x.mask;
            x.mask = mask;
            old
        })
    } else {
        // return existing value
        process_info(|x| x.mask)
    }
}

pub fn set_mask(bitmap: &image::Bitmap, mask: u8) {
    set_bitmap(bitmap, |info| &mut info.bitmap_masks[mask as usize]);
}

pub fn set_bitmap<F>(bitmap: &image::Bitmap, choice: F)
where
    F: FnOnce(&mut info::Info) -> &mut info::BmpArray,
{
    // make 1 pixel per bit into 1 pixel per byte
    let bytes = bitmap.debug_bits().iter().flat_map(|x| {
        let f = |z: Byte| Byte::from(x & (1 << (7 - z)) == 0);
        [f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7)].into_iter()
    });

    process_info(|x| {
        let mut array = choice(x);
        *array = info::BLANK_BMP;
        for (copy, byte) in array.iter_mut().zip(bytes) {
            *copy = byte
        }
    })
}

pub fn set_modes(modes: &Vec<(Mode, String)>) {
    // reduce modes vector into pairs of mode and character byte
    let modes = modes.iter().flat_map(|(m, s)| {
        std::iter::repeat(match m {
            Mode::Numeric => 0,
            Mode::AlphaNum => 1,
            Mode::ASCII => 2,
        })
        .take(s.len())
        .zip(s.bytes())
        .flat_map(|(m, b)| [m, b].into_iter())
    });

    process_info(|x| {
        x.modes = info::BLANK_INFO.modes;
        for (mo, mx) in x.modes.iter_mut().zip(modes) {
            *mo = mx;
        }
    });
}

pub fn reset_all() {
    process_info(|x| {
        x.clear();
    })
}
