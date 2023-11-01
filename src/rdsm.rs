// reed-solomon / galois field operations from wikiversity
// https://en.wikiversity.org/wiki/Reed%E2%80%93Solomon_codes_for_coders
// split out into separate files

mod galois;
mod polynomials;
mod precomputed;

pub(crate) use galois::*;
pub(crate) use polynomials::*;
pub(crate) use precomputed::*;
// use precomputed::PC_EXP_LOG_TABLE;
