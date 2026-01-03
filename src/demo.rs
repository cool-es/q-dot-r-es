//! Data structures primarily for tracking stats for an interactive WASM demo.
#![allow(unused)]

// define types for use here

// javascript-compatible integer
#[cfg(target_arch = "wasm32")]
type NativeInt = i32;

#[cfg(not(target_arch = "wasm32"))]
type NativeInt = usize;

// #[cfg(target_arch = "wasm32")]
mod info {
    struct Info {
        bitmap: Vec<u8>,
        mask: u8,
    }

    static mut INFO_STATE: Info = Info {
        bitmap: Vec::new(),
        mask: u8::MAX,
    };

    static mut BITMAP: Vec<u8> = Vec::new();

    #[allow(static_mut_refs)]
    fn with_info_state<F, R>(func: F) -> R
    where
        F: FnOnce(&mut Info) -> R,
    {
        unsafe { func(&mut INFO_STATE) }
    }

    pub mod ops {
        use super::*;

        pub fn mask(mask: u8) {
            with_info_state(|x| x.mask = mask);
        }
    }
}

// #[cfg(not(target_arch = "wasm32"))]
#[cfg(false)]
mod info {
    use std::cell::RefCell;

    thread_local! {
        // pub static CODEWORDS: RefCell<Vec<(String, u8)>> = RefCell::new(Vec::new());
        // pub static VERSION_INFO: RefCell<Option<u32>> = RefCell::new(None);
        // pub static VERSION: RefCell<Option<u8>> = RefCell::new(None);
        pub static BITMAP: RefCell<Vec<u8>> = RefCell::new(Vec::new());
        pub static MASK: RefCell<Option<u8>> = RefCell::new(None);
    }

    pub mod ops {
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

pub use info::ops;
