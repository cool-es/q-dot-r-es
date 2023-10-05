pub fn alignment_pattern_coords(version: u32) -> Vec<(usize, usize)> {
    if !(1..=40).contains(&version) {
        panic!()
    }
    let indices = AP_COORD_INDICES[version as usize - 1];
    let mut output = Vec::new();

    for &x in indices {
        for &y in indices {
            if [x, y].contains(&6) {
                if x == y || [x, y].contains(&indices.last().unwrap()) {
                    continue;
                }
            }
            output.push((x, y));
        }
    }
    output
}

pub(super) const AP_COORD_INDICES: [&[usize]; 40] = [
    &[],
    &[6, 18],
    &[6, 22],
    &[6, 26],
    &[6, 30],
    &[6, 34],
    &[6, 22, 38],
    &[6, 24, 42],
    &[6, 26, 46],
    &[6, 28, 50],
    &[6, 30, 54],
    &[6, 32, 58],
    &[6, 34, 62],
    &[6, 26, 46, 66],
    &[6, 26, 48, 70],
    &[6, 26, 50, 74],
    &[6, 30, 54, 78],
    &[6, 30, 56, 82],
    &[6, 30, 58, 86],
    &[6, 34, 62, 90],
    &[6, 28, 50, 72, 94],
    &[6, 26, 50, 74, 98],
    &[6, 30, 54, 78, 102],
    &[6, 28, 54, 80, 106],
    &[6, 32, 58, 84, 110],
    &[6, 30, 58, 86, 114],
    &[6, 34, 62, 90, 118],
    &[6, 26, 50, 74, 98, 122],
    &[6, 30, 54, 78, 102, 126],
    &[6, 26, 52, 78, 104, 130],
    &[6, 30, 56, 82, 108, 134],
    &[6, 34, 60, 86, 112, 138],
    &[6, 30, 58, 86, 114, 142],
    &[6, 34, 62, 90, 118, 146],
    &[6, 30, 54, 78, 102, 126, 150],
    &[6, 24, 50, 76, 102, 128, 154],
    &[6, 28, 54, 80, 106, 132, 158],
    &[6, 32, 58, 84, 110, 136, 162],
    &[6, 26, 54, 82, 110, 138, 166],
    &[6, 30, 58, 86, 114, 142, 170],
];

mod _old {
    // coordinates for centers of alignment patterns

    // alignment pattern coord 0 is always 6

    // alignment pattern coord 1, for symbols of version ≥2
    pub const AP_COORDS_1: &[usize] = &[
        18, 22, 26, 30, 34, 22, 24, 26, 28, 30, 32, 34, 26, 26, 26, 30, 30, 30, 34, 28, 26, 30, 28,
        32, 30, 34, 26, 30, 26, 30, 34, 30, 34, 30, 24, 28, 32, 26, 30,
    ];

    // alignment pattern coord 2, for symbols of version ≥7
    pub const AP_COORDS_2: &[usize] = &[
        38, 42, 46, 50, 54, 58, 62, 46, 48, 50, 54, 56, 58, 62, 50, 50, 54, 54, 58, 58, 62, 50, 54,
        52, 56, 60, 58, 62, 54, 50, 54, 58, 54, 58,
    ];

    // alignment pattern coord 3, for symbols of version ≥14
    pub const AP_COORDS_3: &[usize] = &[
        66, 70, 74, 78, 82, 86, 90, 72, 74, 78, 80, 84, 86, 90, 74, 78, 78, 82, 86, 86, 90, 78, 76,
        80, 84, 82, 86,
    ];

    // alignment pattern coord 4, for symbols of version ≥21
    pub const AP_COORDS_4: &[usize] = &[
        94, 98, 102, 106, 110, 114, 118, 98, 102, 104, 108, 112, 114, 118, 102, 102, 106, 110, 110,
        114,
    ];

    // alignment pattern coord 5, for symbols of version ≥28
    pub const AP_COORDS_5: &[usize] = &[
        122, 126, 130, 134, 138, 142, 146, 126, 128, 132, 136, 138, 142,
    ];

    // alignment pattern coord 6, for symbols of version ≥35
    pub const AP_COORDS_6: &[usize] = &[150, 154, 158, 162, 166, 170];

    pub fn version_coordinates(version: u32) -> Vec<usize> {
        if !((1..=40).contains(&version)) {
            panic!()
        }
        let mut output = Vec::new();
        if version > 1 {
            output.push(6);
        }
        for (list, v) in [
            AP_COORDS_1,
            AP_COORDS_2,
            AP_COORDS_3,
            AP_COORDS_4,
            AP_COORDS_5,
            AP_COORDS_6,
        ]
        .iter()
        .zip([2, 7, 14, 21, 28, 35])
        {
            if version >= v {
                output.push(list[(version - v) as usize]);
            } else {
                break;
            }
        }
        output
    }

    pub fn print_table() {
        print!("const DATA: [&[usize];40] = [");
        for i in 1..=40 {
            let a = version_coordinates(i);
            print!("&{:?},", a);
        }
        println!("];");
    }
}
