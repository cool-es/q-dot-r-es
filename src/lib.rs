//! # The *"Q-dot-R-es"* QR code generator

pub mod image;
pub mod qr_standard;
pub mod rdsm;

pub use qr_standard::Mode::*;
pub use qr_standard::*;
