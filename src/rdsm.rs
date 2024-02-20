//! Reed-Solomon and Galois field operations.
//!
//! Partially adapted from [Wikiversity](https://en.wikiversity.org/wiki/Reed%E2%80%93Solomon_codes_for_coders)'s guide, and partially original.

/// Operations on the finite field GF(2⁸).
pub mod galois;
/// Operations on the polynomial ring GF(2⁸)\[X\].
pub mod poly;
/// Precomputed look-up tables.
pub mod lookup;
