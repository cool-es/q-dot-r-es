// i don't know exactly what this code does
pub fn apply_mask(pattern: &Vec<Vec<bool>>, mask: u32) -> Vec<Vec<bool>> {
    let mut masked: Vec<Vec<bool>> = Vec::new();

    for (i, line) in pattern.iter().enumerate() {
        masked.push(Vec::new());
        for (j, pixel) in line.iter().enumerate() {
            masked[i].push(
                pixel
                    ^ match mask {
                        0 => (i + j) % 2 == 0,
                        1 => i % 2 == 0,
                        2 => j % 3 == 0,
                        3 => (i + j) % 3 == 0,
                        4 => (i / 2 + j / 3) % 2 == 0,
                        5 => (i * j) % 2 + (i * j) % 3 == 0,
                        6 => ((i * j) % 3 + i * j) % 2 == 0,
                        7 => ((i * j) % 3 + i + j) % 2 == 0,
                        _ => panic!("invalid bit mask id"),
                    },
            );
        }
    }
    return masked;
}

// generate a blank template of the bits unchanged
// by the bitmask
pub fn generate_qr() -> Vec<Vec<Option<bool>>> {
    todo!();
}
