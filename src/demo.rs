//! Data structures primarily for tracking stats for an interactive WASM demo.
#![allow(unused)]

// define types for use here

// javascript-compatible integer
#[cfg(target_arch = "wasm32")]
type NativeInt = i32;

#[cfg(not(target_arch = "wasm32"))]
type NativeInt = usize;

#[cfg(target_arch = "wasm32")]
mod info {
    use crate::demo::NativeInt;

    struct Info {
        bitmap: Vec<u8>,
        mask: u8,
    }

    static mut INFO_STATE: Info = Info {
        bitmap: Vec::new(),
        mask: u8::MAX,
    };

    #[allow(static_mut_refs)]
    fn process_info<F, K>(f: F) -> K
    where
        F: FnOnce(&mut Info) -> K,
    {
        unsafe { f(&mut INFO_STATE) }
    }

    fn ptr_and_len(ptr: bool, items: &Vec<u8>) -> NativeInt {
        if ptr {
            items.as_ptr() as NativeInt
        } else {
            items.len() as NativeInt
        }
    }

    pub mod ops {
        use crate::demo::NativeInt;

        pub fn mask(mask: Option<u8>) -> u8 {
            if let Some(mask) = mask {
                // set new value
                process_info(|x| {
                    let old = *x.mask;
                    x.mask = mask;
                    old
                })
            } else {
                // return existing value
                process_info(|x| *x.mask)
            }
        }

        pub fn reset_all() {
            process_info(|x| {
                *x = Info {
                    bitmap: Vec::new(),
                    mask: u8::MAX,
                }
            })
        }

        pub fn bitmap(k: bool) -> NativeInt {
            process_info(|x| ptr_and_len(k, &x.bitmap))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod info {
    use crate::demo::NativeInt;
    use std::cell::RefCell;

    thread_local! {
        // pub static CODEWORDS: RefCell<Vec<(String, u8)>> = RefCell::new(Vec::new());
        // pub static VERSION_INFO: RefCell<Option<u32>> = RefCell::new(None);
        // pub static VERSION: RefCell<Option<u8>> = RefCell::new(None);
        pub static BITMAP: RefCell<Vec<u8>> = RefCell::new(Vec::new());
        pub static MASK: RefCell<Option<u8>> = RefCell::new(None);
    }

    pub mod ops {
        use super::*;
        use crate::demo::NativeInt;

        pub fn set_mask(mask: u8) {
            MASK.with(|m| *m.borrow_mut() = Some(mask));
        }
    }

    // pub fn set_version(version: u8) {
    //     VERSION.with(|v| *v.borrow_mut() = Some(version));
    // }

    // pub fn set_codewords(codewords: Vec<(String, u8)>) {
    //     CODEWORDS.with(|c| *c.borrow_mut() = codewords);
    // }

    // Getters
    // pub fn get_mask() -> Option<u8> {
    //     MASK.with(|m| *m.borrow())
    // }

    // etc...
}

use info::*;
