//! operations to be called by the end user

use crate::{
    demo::{info, Byte, NativeInt},
    image,
    qr_standard::bitstream::{Mode, Token},
};

// the specific static variable storing the info at a specific place in memory
static mut INFO_STATE: info::Info = info::Info::new();

// the unsafe "swiss army knife function"
#[allow(static_mut_refs)]
pub(crate) fn with_info<F, K>(f: F) -> K
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
        with_info(|x| {
            let old = x.mask;
            x.mask = mask;
            old
        })
    } else {
        // return existing value
        with_info(|x| x.mask)
    }
}

pub fn set_bitmap<B, F>(bitmap: B, choice: F)
where
    B: std::borrow::Borrow<image::Bitmap>,
    F: FnOnce(&mut info::Info) -> &mut info::BmpArray,
{
    // make 1 pixel per bit into 1 pixel per byte
    let bytes = bitmap
        .borrow()
        .debug_bits()
        .iter()
        .flat_map(|x| {
            let f = |z: Byte| Byte::from(x & (1 << (7 - z)) == 0);
            [f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7)].into_iter()
        })
        .collect::<Vec<Byte>>();

    with_info(|info| {
        let array = choice(info);
        *array = info::BLANK_BMP;
        array[..bytes.len()].copy_from_slice(&bytes);
    })
}

pub fn set_modes<T, S>(modes: T)
where
    T: AsRef<[(Mode, S)]>,
    S: AsRef<[u8]>,
{
    // reduce modes vector into pairs of mode and character byte
    let mode_slice: Vec<u8> = modes
        .as_ref()
        .iter()
        .flat_map(|(m, s)| {
            let s: &[u8] = s.as_ref();
            std::iter::repeat_n(Mode::demo_index(*m), s.len())
                .zip(s.iter())
                .flat_map(|(m, b)| [m as u8, *b].into_iter())
        })
        .collect();

    with_info(|x| {
        x.modes = info::BLANK_INFO.modes;
        let min = mode_slice.len().min(x.modes.len());
        x.modes[..min].copy_from_slice(&mode_slice[..min]);
    });
}

pub fn reset_all() {
    with_info(|x| {
        x.clear();
    })
}
