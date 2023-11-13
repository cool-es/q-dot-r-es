use super::*;

/// The non-zero elements of GF(2⁸). Lookup table for [exp] and [log].
pub(crate) const QR_EXP_LOG_TABLE: ExpLogLUTs = (
    [
        // 255 values of usize -> element
        0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1D, 0x3A, 0x74, 0xE8, 0xCD, 0x87, 0x13,
        0x26, 0x4C, 0x98, 0x2D, 0x5A, 0xB4, 0x75, 0xEA, 0xC9, 0x8F, 0x03, 0x06, 0x0C, 0x18, 0x30,
        0x60, 0xC0, 0x9D, 0x27, 0x4E, 0x9C, 0x25, 0x4A, 0x94, 0x35, 0x6A, 0xD4, 0xB5, 0x77, 0xEE,
        0xC1, 0x9F, 0x23, 0x46, 0x8C, 0x05, 0x0A, 0x14, 0x28, 0x50, 0xA0, 0x5D, 0xBA, 0x69, 0xD2,
        0xB9, 0x6F, 0xDE, 0xA1, 0x5F, 0xBE, 0x61, 0xC2, 0x99, 0x2F, 0x5E, 0xBC, 0x65, 0xCA, 0x89,
        0x0F, 0x1E, 0x3C, 0x78, 0xF0, 0xFD, 0xE7, 0xD3, 0xBB, 0x6B, 0xD6, 0xB1, 0x7F, 0xFE, 0xE1,
        0xDF, 0xA3, 0x5B, 0xB6, 0x71, 0xE2, 0xD9, 0xAF, 0x43, 0x86, 0x11, 0x22, 0x44, 0x88, 0x0D,
        0x1A, 0x34, 0x68, 0xD0, 0xBD, 0x67, 0xCE, 0x81, 0x1F, 0x3E, 0x7C, 0xF8, 0xED, 0xC7, 0x93,
        0x3B, 0x76, 0xEC, 0xC5, 0x97, 0x33, 0x66, 0xCC, 0x85, 0x17, 0x2E, 0x5C, 0xB8, 0x6D, 0xDA,
        0xA9, 0x4F, 0x9E, 0x21, 0x42, 0x84, 0x15, 0x2A, 0x54, 0xA8, 0x4D, 0x9A, 0x29, 0x52, 0xA4,
        0x55, 0xAA, 0x49, 0x92, 0x39, 0x72, 0xE4, 0xD5, 0xB7, 0x73, 0xE6, 0xD1, 0xBF, 0x63, 0xC6,
        0x91, 0x3F, 0x7E, 0xFC, 0xE5, 0xD7, 0xB3, 0x7B, 0xF6, 0xF1, 0xFF, 0xE3, 0xDB, 0xAB, 0x4B,
        0x96, 0x31, 0x62, 0xC4, 0x95, 0x37, 0x6E, 0xDC, 0xA5, 0x57, 0xAE, 0x41, 0x82, 0x19, 0x32,
        0x64, 0xC8, 0x8D, 0x07, 0x0E, 0x1C, 0x38, 0x70, 0xE0, 0xDD, 0xA7, 0x53, 0xA6, 0x51, 0xA2,
        0x59, 0xB2, 0x79, 0xF2, 0xF9, 0xEF, 0xC3, 0x9B, 0x2B, 0x56, 0xAC, 0x45, 0x8A, 0x09, 0x12,
        0x24, 0x48, 0x90, 0x3D, 0x7A, 0xF4, 0xF5, 0xF7, 0xF3, 0xFB, 0xEB, 0xCB, 0x8B, 0x0B, 0x16,
        0x2C, 0x58, 0xB0, 0x7D, 0xFA, 0xE9, 0xCF, 0x83, 0x1B, 0x36, 0x6C, 0xD8, 0xAD, 0x47, 0x8E,
    ],
    [
        // 255 values of element -> usize
        0x00, 0x01, 0x19, 0x02, 0x32, 0x1A, 0xC6, 0x03, 0xDF, 0x33, 0xEE, 0x1B, 0x68, 0xC7, 0x4B,
        0x04, 0x64, 0xE0, 0x0E, 0x34, 0x8D, 0xEF, 0x81, 0x1C, 0xC1, 0x69, 0xF8, 0xC8, 0x08, 0x4C,
        0x71, 0x05, 0x8A, 0x65, 0x2F, 0xE1, 0x24, 0x0F, 0x21, 0x35, 0x93, 0x8E, 0xDA, 0xF0, 0x12,
        0x82, 0x45, 0x1D, 0xB5, 0xC2, 0x7D, 0x6A, 0x27, 0xF9, 0xB9, 0xC9, 0x9A, 0x09, 0x78, 0x4D,
        0xE4, 0x72, 0xA6, 0x06, 0xBF, 0x8B, 0x62, 0x66, 0xDD, 0x30, 0xFD, 0xE2, 0x98, 0x25, 0xB3,
        0x10, 0x91, 0x22, 0x88, 0x36, 0xD0, 0x94, 0xCE, 0x8F, 0x96, 0xDB, 0xBD, 0xF1, 0xD2, 0x13,
        0x5C, 0x83, 0x38, 0x46, 0x40, 0x1E, 0x42, 0xB6, 0xA3, 0xC3, 0x48, 0x7E, 0x6E, 0x6B, 0x3A,
        0x28, 0x54, 0xFA, 0x85, 0xBA, 0x3D, 0xCA, 0x5E, 0x9B, 0x9F, 0x0A, 0x15, 0x79, 0x2B, 0x4E,
        0xD4, 0xE5, 0xAC, 0x73, 0xF3, 0xA7, 0x57, 0x07, 0x70, 0xC0, 0xF7, 0x8C, 0x80, 0x63, 0x0D,
        0x67, 0x4A, 0xDE, 0xED, 0x31, 0xC5, 0xFE, 0x18, 0xE3, 0xA5, 0x99, 0x77, 0x26, 0xB8, 0xB4,
        0x7C, 0x11, 0x44, 0x92, 0xD9, 0x23, 0x20, 0x89, 0x2E, 0x37, 0x3F, 0xD1, 0x5B, 0x95, 0xBC,
        0xCF, 0xCD, 0x90, 0x87, 0x97, 0xB2, 0xDC, 0xFC, 0xBE, 0x61, 0xF2, 0x56, 0xD3, 0xAB, 0x14,
        0x2A, 0x5D, 0x9E, 0x84, 0x3C, 0x39, 0x53, 0x47, 0x6D, 0x41, 0xA2, 0x1F, 0x2D, 0x43, 0xD8,
        0xB7, 0x7B, 0xA4, 0x76, 0xC4, 0x17, 0x49, 0xEC, 0x7F, 0x0C, 0x6F, 0xF6, 0x6C, 0xA1, 0x3B,
        0x52, 0x29, 0x9D, 0x55, 0xAA, 0xFB, 0x60, 0x86, 0xB1, 0xBB, 0xCC, 0x3E, 0x5A, 0xCB, 0x59,
        0x5F, 0xB0, 0x9C, 0xA9, 0xA0, 0x51, 0x0B, 0xF5, 0x16, 0xEB, 0x7A, 0x75, 0x2C, 0xD7, 0x4F,
        0xAE, 0xD5, 0xE9, 0xE6, 0xE7, 0xAD, 0xE8, 0x74, 0xD6, 0xF4, 0xEA, 0xA8, 0x50, 0x58, 0xAF,
    ],
);

/// All possible Reed-Solomon generator polynomials used for QR codes, for [encode_message].
///
/// All values coincide with the ones given in the standards document.
pub(crate) const RDSM_GENERATOR_POLYNOMIALS: [&[Element]; 31] = [
    &[0x01, 0x7F, 0x7A, 0x9A, 0xA4, 0x0B, 0x44, 0x75],
    &[
        0x01, 0xD8, 0xC2, 0x9F, 0x6F, 0xC7, 0x5E, 0x5F, 0x71, 0x9D, 0xC1,
    ],
    &[
        0x01, 0x89, 0x49, 0xE3, 0x11, 0xB1, 0x11, 0x34, 0x0D, 0x2E, 0x2B, 0x53, 0x84, 0x78,
    ],
    &[
        0x01, 0x1D, 0xC4, 0x6F, 0xA3, 0x70, 0x4A, 0x0A, 0x69, 0x69, 0x8B, 0x84, 0x97, 0x20, 0x86,
        0x1A,
    ],
    &[
        0x01, 0x3B, 0x0D, 0x68, 0xBD, 0x44, 0xD1, 0x1E, 0x08, 0xA3, 0x41, 0x29, 0xE5, 0x62, 0x32,
        0x24, 0x3B,
    ],
    &[
        0x01, 0x77, 0x42, 0x53, 0x78, 0x77, 0x16, 0xC5, 0x53, 0xF9, 0x29, 0x8F, 0x86, 0x55, 0x35,
        0x7D, 0x63, 0x4F,
    ],
    &[
        0x01, 0xEF, 0xFB, 0xB7, 0x71, 0x95, 0xAF, 0xC7, 0xD7, 0xF0, 0xDC, 0x49, 0x52, 0xAD, 0x4B,
        0x20, 0x43, 0xD9, 0x92,
    ],
    &[
        0x01, 0x98, 0xB9, 0xF0, 0x05, 0x6F, 0x63, 0x06, 0xDC, 0x70, 0x96, 0x45, 0x24, 0xBB, 0x16,
        0xE4, 0xC6, 0x79, 0x79, 0xA5, 0xAE,
    ],
    &[
        0x01, 0x59, 0xB3, 0x83, 0xB0, 0xB6, 0xF4, 0x13, 0xBD, 0x45, 0x28, 0x1C, 0x89, 0x1D, 0x7B,
        0x43, 0xFD, 0x56, 0xDA, 0xE6, 0x1A, 0x91, 0xF5,
    ],
    &[
        0x01, 0x7A, 0x76, 0xA9, 0x46, 0xB2, 0xED, 0xD8, 0x66, 0x73, 0x96, 0xE5, 0x49, 0x82, 0x48,
        0x3D, 0x2B, 0xCE, 0x01, 0xED, 0xF7, 0x7F, 0xD9, 0x90, 0x75,
    ],
    &[
        0x01, 0xF6, 0x33, 0xB7, 0x04, 0x88, 0x62, 0xC7, 0x98, 0x4D, 0x38, 0xCE, 0x18, 0x91, 0x28,
        0xD1, 0x75, 0xE9, 0x2A, 0x87, 0x44, 0x46, 0x90, 0x92, 0x4D, 0x2B, 0x5E,
    ],
    &[
        0x01, 0xFC, 0x09, 0x1C, 0x0D, 0x12, 0xFB, 0xD0, 0x96, 0x67, 0xAE, 0x64, 0x29, 0xA7, 0x0C,
        0xF7, 0x38, 0x75, 0x77, 0xE9, 0x7F, 0xB5, 0x64, 0x79, 0x93, 0xB0, 0x4A, 0x3A, 0xC5,
    ],
    &[
        0x01, 0xD4, 0xF6, 0x4D, 0x49, 0xC3, 0xC0, 0x4B, 0x62, 0x05, 0x46, 0x67, 0xB1, 0x16, 0xD9,
        0x8A, 0x33, 0xB5, 0xF6, 0x48, 0x19, 0x12, 0x2E, 0xE4, 0x4A, 0xD8, 0xC3, 0x0B, 0x6A, 0x82,
        0x96,
    ],
    &[
        0x01, 0x74, 0x40, 0x34, 0xAE, 0x36, 0x7E, 0x10, 0xC2, 0xA2, 0x21, 0x21, 0x9D, 0xB0, 0xC5,
        0xE1, 0x0C, 0x3B, 0x37, 0xFD, 0xE4, 0x94, 0x2F, 0xB3, 0xB9, 0x18, 0x8A, 0xFD, 0x14, 0x8E,
        0x37, 0xAC, 0x58,
    ],
    &[
        0x01, 0xCE, 0x3C, 0x9A, 0x71, 0x06, 0x75, 0xD0, 0x5A, 0x1A, 0x71, 0x1F, 0x19, 0xB1, 0x84,
        0x63, 0x33, 0x69, 0xB7, 0x7A, 0x16, 0x2B, 0x88, 0x5D, 0x5E, 0x3E, 0x6F, 0xC4, 0x17, 0x7E,
        0x87, 0x43, 0xDE, 0x17, 0x0A,
    ],
    &[
        0x01, 0x1C, 0xC4, 0x43, 0x4C, 0x7B, 0xC0, 0xCF, 0xFB, 0xB9, 0x49, 0x7C, 0x01, 0x7E, 0x49,
        0x1F, 0x1B, 0x0B, 0x68, 0x2D, 0xA1, 0x2B, 0x4A, 0x7F, 0x59, 0x1A, 0xDB, 0x3B, 0x89, 0x76,
        0xC8, 0xED, 0xD8, 0x1F, 0xF3, 0x60, 0x3B,
    ],
    &[
        0x01, 0xD2, 0xF8, 0xF0, 0xD1, 0xAD, 0x43, 0x85, 0xA7, 0x85, 0xD1, 0x83, 0xBA, 0x63, 0x5D,
        0xEB, 0x34, 0x28, 0x06, 0xDC, 0xF1, 0x48, 0x0D, 0xD7, 0x80, 0xFF, 0x9C, 0x31, 0x3E, 0xFE,
        0xD4, 0x23, 0x63, 0x33, 0xDA, 0x65, 0xB4, 0xF7, 0x28, 0x9C, 0x26,
    ],
    &[
        0x01, 0x6C, 0x88, 0x45, 0xF4, 0x03, 0x2D, 0x9E, 0xF5, 0x01, 0x08, 0x69, 0xB0, 0x45, 0x41,
        0x67, 0x6B, 0xF4, 0x1D, 0xA5, 0x34, 0xD9, 0x29, 0x26, 0x5C, 0x42, 0x4E, 0x22, 0x09, 0x35,
        0x22, 0xF2, 0x0E, 0x8B, 0x8E, 0x38, 0xC5, 0xB3, 0xBF, 0x32, 0xED, 0x05, 0xD9,
    ],
    &[
        0x01, 0xAE, 0x80, 0x6F, 0x76, 0xBC, 0xCF, 0x2F, 0xA0, 0xFC, 0xA5, 0xE1, 0x7D, 0x41, 0x03,
        0x65, 0xC5, 0x3A, 0x4D, 0x13, 0x83, 0x02, 0x0B, 0xEE, 0x78, 0x54, 0xDE, 0x12, 0x66, 0xC7,
        0x3E, 0x99, 0x63, 0x14, 0x32, 0x9B, 0x29, 0xDD, 0xE5, 0x4A, 0x2E, 0x1F, 0x44, 0xCA, 0x31,
    ],
    &[
        0x01, 0x81, 0x71, 0xFE, 0x81, 0x47, 0x12, 0x70, 0x7C, 0xDC, 0x86, 0xE1, 0x20, 0x50, 0x1F,
        0x17, 0xEE, 0x69, 0x4C, 0xA9, 0xC3, 0xE5, 0xB2, 0x25, 0x02, 0x10, 0xD9, 0xB9, 0x58, 0xCA,
        0x0D, 0xFB, 0x1D, 0x36, 0xE9, 0x93, 0xF1, 0x14, 0x03, 0xD5, 0x12, 0x77, 0x70, 0x09, 0x5A,
        0xD3, 0x26,
    ],
    &[
        0x01, 0x3D, 0x03, 0xC8, 0x2E, 0xB2, 0x9A, 0xB9, 0x8F, 0xD8, 0xDF, 0x35, 0x44, 0x2C, 0x6F,
        0xAB, 0xA1, 0x9F, 0xC5, 0x7C, 0x2D, 0x45, 0xCE, 0xA9, 0xE6, 0x62, 0xA7, 0x68, 0x53, 0xE2,
        0x55, 0x3B, 0x95, 0xA3, 0x75, 0x83, 0xE4, 0x84, 0x0B, 0x41, 0xE8, 0x71, 0x90, 0x6B, 0x05,
        0x63, 0x35, 0x4E, 0xD0,
    ],
    &[
        0x01, 0xF7, 0x33, 0xD5, 0xD1, 0xC6, 0x3A, 0xC7, 0x9F, 0xA2, 0x86, 0xE0, 0x19, 0x9C, 0x08,
        0xA2, 0xCE, 0x64, 0xB0, 0xE0, 0x24, 0x9F, 0x87, 0x9D, 0xE6, 0x66, 0xA2, 0x2E, 0xE6, 0xB0,
        0xEF, 0xB0, 0x0F, 0x3C, 0xB5, 0x57, 0x9D, 0x1F, 0xBE, 0x97, 0x2F, 0x3D, 0x3E, 0xEB, 0xFF,
        0x97, 0xD7, 0xEF, 0xF7, 0x6D, 0xA7,
    ],
    &[
        0x01, 0xF8, 0x05, 0xB1, 0x6E, 0x05, 0xAC, 0xD8, 0xE1, 0x82, 0x9F, 0xB1, 0xCC, 0x97, 0x5A,
        0x95, 0xF3, 0xAA, 0xEF, 0xEA, 0x13, 0xD2, 0x4D, 0x4A, 0xB0, 0xE0, 0xDA, 0x8E, 0xE1, 0xAE,
        0x71, 0xD2, 0xBE, 0x97, 0x1F, 0x11, 0xF3, 0xEB, 0x76, 0xEA, 0x1E, 0xB1, 0xAF, 0x35, 0xB0,
        0x1C, 0xAC, 0x22, 0x27, 0x16, 0x8E, 0xF8, 0x0A,
    ],
    &[
        0x01, 0xC4, 0x06, 0x38, 0x7F, 0x59, 0x45, 0x1F, 0x75, 0x9F, 0xBE, 0xC1, 0x05, 0x0B, 0x95,
        0x36, 0x24, 0x44, 0x69, 0xA2, 0x2B, 0xBD, 0x91, 0x06, 0xE2, 0x95, 0x82, 0x14, 0xE9, 0x9C,
        0x8E, 0x0B, 0xFF, 0x7B, 0xF0, 0xC5, 0x03, 0xEC, 0x77, 0x3B, 0xD0, 0xEF, 0xFD, 0x85, 0x38,
        0xEB, 0x1D, 0x92, 0xD2, 0x22, 0xC0, 0x07, 0x1E, 0xC0, 0xE4,
    ],
    &[
        0x01, 0x34, 0x3B, 0x68, 0xD5, 0xC6, 0xC3, 0x81, 0xF8, 0x04, 0xA3, 0x1B, 0x63, 0x25, 0x38,
        0x70, 0x7A, 0x40, 0xA8, 0x8E, 0x72, 0xA9, 0x51, 0xD7, 0xA2, 0xCD, 0x42, 0xCC, 0x2A, 0x62,
        0x36, 0xDB, 0xF1, 0xAE, 0x18, 0x74, 0xD6, 0x16, 0x95, 0x22, 0x97, 0x49, 0x53, 0xD9, 0xC9,
        0x63, 0x6F, 0x0C, 0xC8, 0x83, 0xAA, 0x39, 0x70, 0xA6, 0xB4, 0x6F, 0x74,
    ],
    &[
        0x01, 0xD3, 0xF8, 0x06, 0x83, 0x61, 0x0C, 0xDE, 0x68, 0xAD, 0x62, 0x1C, 0x37, 0xEB, 0xA0,
        0xD8, 0xB0, 0x59, 0xA8, 0x39, 0x8B, 0xE3, 0x15, 0x82, 0x1B, 0x49, 0x36, 0x53, 0xD6, 0x47,
        0x2A, 0xBE, 0x91, 0x33, 0xC9, 0x8F, 0x60, 0xEC, 0x2C, 0xF9, 0x40, 0x17, 0x2B, 0x30, 0x4D,
        0xCC, 0xDA, 0x53, 0xE9, 0xED, 0x30, 0xD4, 0xA1, 0x73, 0x2A, 0xF3, 0x33, 0x52, 0xC5,
    ],
    &[
        0x01, 0x68, 0x84, 0x06, 0xCD, 0x3A, 0x15, 0x7D, 0x8D, 0x48, 0x8D, 0x56, 0xC1, 0xB2, 0x22,
        0x56, 0x3B, 0x18, 0x31, 0xCC, 0x40, 0x11, 0x83, 0x04, 0xA7, 0x07, 0xBA, 0x7C, 0x56, 0x22,
        0xBD, 0xE6, 0xD3, 0x4A, 0x94, 0x0B, 0x8C, 0xE6, 0xA2, 0x76, 0xB1, 0xE8, 0x97, 0x60, 0x31,
        0x6B, 0x03, 0x32, 0x7F, 0xBE, 0x44, 0xAE, 0xAC, 0x5E, 0x0C, 0xA2, 0x4C, 0xE1, 0x80, 0x27,
        0x2C,
    ],
    &[
        0x01, 0xBE, 0x70, 0x1F, 0x43, 0xBC, 0x09, 0x1B, 0xC7, 0xF9, 0x71, 0x01, 0xEC, 0x4A, 0xC9,
        0x04, 0x3D, 0x69, 0x76, 0x80, 0x1A, 0xA9, 0x78, 0x7D, 0xC7, 0x5E, 0x1E, 0x09, 0xE1, 0x65,
        0x05, 0x5E, 0xCE, 0x32, 0x98, 0x79, 0x66, 0x31, 0x9C, 0x45, 0xED, 0xEB, 0xE8, 0x7A, 0xA4,
        0x29, 0xC5, 0xF2, 0x6A, 0x7C, 0x40, 0x1C, 0x11, 0x06, 0xCF, 0x62, 0x2B, 0xCC, 0xEF, 0x25,
        0x6E, 0x67, 0x34,
    ],
    &[
        0x01, 0xC1, 0x0A, 0xFF, 0x3A, 0x80, 0xB7, 0x73, 0x8C, 0x99, 0x93, 0x5B, 0xC5, 0xDB, 0xDD,
        0xDC, 0x8E, 0x1C, 0x78, 0x15, 0xA4, 0x93, 0x06, 0xCC, 0x28, 0xE6, 0xB6, 0x0E, 0x79, 0x30,
        0x8F, 0x4D, 0xE4, 0x51, 0x55, 0x2B, 0xA2, 0x10, 0xC3, 0xA3, 0x23, 0x95, 0x9A, 0x23, 0x84,
        0x64, 0x64, 0x33, 0xB0, 0x0B, 0xA1, 0x86, 0xD0, 0x84, 0xF4, 0xB0, 0xC0, 0xDD, 0xE8, 0xAB,
        0x7D, 0x9B, 0xE4, 0xF2, 0xF5,
    ],
    &[
        0x01, 0x20, 0xC7, 0x8A, 0x96, 0x4F, 0x4F, 0xBF, 0x0A, 0x9F, 0xED, 0x87, 0xEF, 0xE7, 0x98,
        0x42, 0x83, 0x8D, 0xB3, 0xE2, 0xF6, 0xBE, 0x9E, 0xAB, 0x99, 0xCE, 0xE2, 0x22, 0xD4, 0x65,
        0xF9, 0xE5, 0x8D, 0xE2, 0x80, 0xEE, 0x39, 0x3C, 0xCE, 0xCB, 0x6A, 0x76, 0x54, 0xA1, 0x7F,
        0xFD, 0x47, 0x2C, 0x66, 0x9B, 0x3C, 0x4E, 0xF7, 0x34, 0x05, 0xFC, 0xD3, 0x1E, 0x9A, 0xC2,
        0x34, 0xB3, 0x03, 0xB8, 0xB6, 0xC1, 0x1A,
    ],
    &[
        0x01, 0x83, 0x73, 0x09, 0x27, 0x12, 0xB6, 0x3C, 0x5E, 0xDF, 0xE6, 0x9D, 0x8E, 0x77, 0x55,
        0x6B, 0x22, 0xAE, 0xA7, 0x6D, 0x14, 0xB9, 0x70, 0x91, 0xAC, 0xE0, 0xAA, 0xB6, 0x6B, 0x26,
        0x6B, 0x47, 0xF6, 0xE6, 0xE1, 0x90, 0x14, 0x0E, 0xAF, 0xE2, 0xF5, 0x14, 0xDB, 0xD4, 0x33,
        0x9E, 0x58, 0x3F, 0x24, 0xC7, 0x04, 0x50, 0x9D, 0xD3, 0xEF, 0xFF, 0x07, 0x77, 0x0B, 0xEB,
        0x0C, 0x22, 0x95, 0xCC, 0x08, 0x20, 0x1D, 0x63, 0x0B,
    ],
];
