//! Data structures primarily for tracking stats for an interactive WASM demo.
#![allow(unused)]

// define types for use here

// javascript-compatible integer
#[cfg(target_arch = "wasm32")]
type NativeInt = i32;

#[cfg(not(target_arch = "wasm32"))]
type NativeInt = usize;

type ByteVec = Vec<u8>;

mod info {
    use crate::demo::{ByteVec, NativeInt};
    use crate::qr_standard::bitstream::{Mode, Token};

    // the structure holding information about the qr code
    struct Info {
        bitmap: ByteVec,
        codewords: ByteVec,
        mask: u8,
        version: u8,
    }

    // the specific static variable storing the info at a specific place in memory
    static mut INFO_STATE: Info = Info {
        bitmap: Vec::new(),
        codewords: Vec::new(),
        mask: u8::MAX,
        version: u8::MAX,
    };

    // the unsafe "swiss army knife function"
    #[allow(static_mut_refs)]
    fn process_info<F, K>(f: F) -> K
    where
        F: FnOnce(&mut Info) -> K,
    {
        unsafe { f(&mut INFO_STATE) }
    }

    // returns pointer to and byte length of a vector
    fn ptr_and_len<T>(vec: &'static T) -> (NativeInt, NativeInt)
    where
        ByteVec: From<&'static T>,
    {
        let bytes: ByteVec = vec.into();
        (bytes.as_ptr() as NativeInt, bytes.len() as NativeInt)
    }

    // operations to be called by the end user
    pub mod ops {
        use super::*;
        use crate::demo::NativeInt;

        pub fn mask(mask: Option<u8>) -> u8 {
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

        pub fn bitmap(bitmap: ByteVec) {
            // set new value
            process_info(|x| {
                x.bitmap = bitmap;
            })
        }

        pub fn reset_all() {
            process_info(|x| {
                *x = Info {
                    bitmap: Vec::new(),
                    codewords: Vec::new(),
                    mask: u8::MAX,
                    version: u8::MAX,
                }
            })
        }
    }
}

use info::ops::*;
