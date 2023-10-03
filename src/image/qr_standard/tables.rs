// coordinates for centers of alignment patterns

// alignment pattern coord 0 is always 6

// alignment pattern coord 1, for symbols of version ≥2
pub const AP_COORDS_1: &[usize] = &[
    18, 22, 26, 30, 34, 22, 24, 26, 28, 30, 32, 34, 26, 26, 26, 30, 30, 30, 34, 28, 26, 30, 28, 32,
    30, 34, 26, 30, 26, 30, 34, 30, 34, 30, 24, 28, 32, 26, 30,
];

// alignment pattern coord 2, for symbols of version ≥7
pub const AP_COORDS_2: &[usize] = &[
    38, 42, 46, 50, 54, 58, 62, 46, 48, 50, 54, 56, 58, 62, 50, 50, 54, 54, 58, 58, 62, 50, 54, 52,
    56, 60, 58, 62, 54, 50, 54, 58, 54, 58,
];

// alignment pattern coord 3, for symbols of version ≥14
pub const AP_COORDS_3: &[usize] = &[
    66, 70, 74, 78, 82, 86, 90, 72, 74, 78, 80, 84, 86, 90, 74, 78, 78, 82, 86, 86, 90, 78, 76, 80,
    84, 82, 86,
];

// alignment pattern coord 4, for symbols of version ≥21
pub const AP_COORDS_4: &[usize] = &[
    94, 98, 102, 106, 110, 114, 118, 98, 102, 104, 108, 112, 114, 118, 102, 102, 106, 110, 110, 114,
];

// alignment pattern coord 5, for symbols of version ≥28
pub const AP_COORDS_5: &[usize] = &[
    122, 126, 130, 134, 138, 142, 146, 126, 128, 132, 136, 138, 142,
];

// alignment pattern coord 6, for symbols of version ≥35
pub const AP_COORDS_6: &[usize] = &[150, 154, 158, 162, 166, 170];
