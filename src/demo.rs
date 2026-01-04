//! Data structures primarily for tracking stats for an interactive WASM demo.

#[allow(unused)]
mod info;
pub use info::ops;

// define types for use here

type Byte = u8;

// javascript-compatible integer
#[cfg(target_arch = "wasm32")]
type NativeInt = i32;

#[cfg(not(target_arch = "wasm32"))]
type NativeInt = usize;
