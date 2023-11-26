//! # The *"Q-dot-R-es"* QR code generator
#![allow(dead_code)]

pub mod image;
pub mod qr_standard;
pub mod rdsm;

pub use image::Bitmap;
pub use qr_standard::Mode::*;
pub use rdsm::*;
