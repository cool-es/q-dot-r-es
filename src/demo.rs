//! Data structures primarily for tracking stats for an interactive WASM demo.
#![allow(unused)]

// define types for use here

// javascript-compatible integer
#[cfg(target_arch = "wasm32")]
type NativeInt = i32;

#[cfg(not(target_arch = "wasm32"))]
type NativeInt = usize;

type Byte = u8;

mod info {
    use super::{Byte, NativeInt};
    use crate::qr_standard::bitstream::{Mode, Token};

    // the structure holding information about the qr code
    #[derive(Debug, Clone)]
    struct Info {
        bitmap_nomask: Vec<Byte>,
        bitmap: Vec<Byte>,
        codewords: Vec<Byte>,
        ecblock_data: Vec<Byte>,
        corner_mask_data: Vec<Byte>,
        mask: NativeInt,
        modes: Vec<Byte>,
        version: NativeInt,
    }

    impl Info {
        const fn new() -> Info {
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
    }

    // the specific static variable storing the info at a specific place in memory
    static mut INFO_STATE: Info = Info::new();

    // the unsafe "swiss army knife function"
    #[allow(static_mut_refs)]
    fn process_info<F, K>(f: F) -> K
    where
        F: FnOnce(&mut Info) -> K,
    {
        unsafe { f(&mut INFO_STATE) }
    }

    // returns pointer to and byte length of a vector
    fn ptr_and_len<T>(v: &'static T) -> (NativeInt, NativeInt)
    where
        Vec<u8>: From<&'static T>,
    {
        let bytes: Vec<u8> = v.into();
        (bytes.as_ptr() as NativeInt, bytes.len() as NativeInt)
    }

    // operations to be called by the end user
    pub mod ops {
        use super::*;
        use crate::demo::NativeInt;

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
                *x = Info::new();
            })
        }
    }
}

pub use info::ops;
