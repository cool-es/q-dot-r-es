//! operations to be called by the end user

use crate::{
    demo::{info_struct, Byte, NativeInt},
    qr_standard::bitstream::{Mode, Token},
};

// the specific static variable storing the info at a specific place in memory
static mut INFO_STATE: info_struct::Info = info_struct::Info::new();

// the unsafe "swiss army knife function"
#[allow(static_mut_refs)]
pub(crate) fn process_info<F, K>(f: F) -> K
where
    F: FnOnce(&mut info_struct::Info) -> K,
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

pub fn set_bitmap(bitmap: &Vec<Byte>) {
    process_info(|x| {
        x.bitmap.clear();
        x.bitmap.extend(bitmap.iter());
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

    // static means no drop, so clear out the vector instead of making a new one
    process_info(|x| {
        x.modes.clear();
        x.modes.extend(modes);
    });
}

pub fn reset_all() {
    process_info(|x| {
        x.clear();
    })
}
