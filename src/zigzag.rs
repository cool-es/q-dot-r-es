// change to use "Vec<Vec<bool>>" input/output type, if necessary

// convert vector of bytes into vector of bit arrays
pub fn binarify(bytevec: &Vec<u8>) -> Vec<[bool; 8]> {
    let mut bitvec: Vec<[bool; 8]> = Vec::new();

    for byte in bytevec {
        let mut bits: [bool; 8] = [false; 8];
        
        for id in 0..8 {
            // "7 - id" to adjust for endianness
            bits[7 - id] = (byte & (1 << id)) != 0;
        }
        bitvec.push(bits)
    }
    return bitvec;
}

pub fn test_btb () {
    let bitvec = binarify(&vec![0x40, 0xd2, 0x75, 0x47, 0x76, 0x17, 0x32, 0x06, 0x27, 0x26, 0x96, 0xc6, 0xc6, 0x96, 0x70, 0xec]);

    for byte in bitvec {
        for bit in byte {
            print!("{}", u32::from(bit));
        }
        println!();
    }
}

// takes "curve" 2d vector, returns x-y coordinate pairs
pub fn curve_to_coords(curve: &Vec<Vec<Option<u32>>>) -> Vec<(usize, usize)> {
    let mut coords: Vec<(usize, usize)> = Vec::new();

    for (y, line) in curve.iter().enumerate() {
        for (x, num) in line.iter().enumerate() {
            if *num == None {
                continue;
            } else {
                let n = num.unwrap() as usize;

                if n + 1 > coords.len() {
                    coords.resize(n + 1, (0, 0));
                }

                if coords[n] != (0, 0) {
                    println!(
                        "Curve contains repeated points! {:?} = {:?}",
                        coords[n],
                        (x, y)
                    );
                } else {
                    coords[n] = (x, y);
                }
            }
        }
    }
    return coords;
}

// takes "curve" 2d vector and string slice, returns 2d vector with text
pub fn curve_to_text(curve: &Vec<Vec<Option<u32>>>, string: &str) -> Vec<Vec<Option<char>>> {
    // this function is "naive" and does not check formatting
    let mut output: Vec<Vec<Option<char>>> = Vec::new();
    let mut letvec: Vec<char> = Vec::new();

    for letter in string.chars() {
        letvec.push(letter);
    }

    for (y, line) in curve.iter().enumerate() {
        //if y+1 > output.len() {
        //    output.resize(y+1, Vec::new());
        //}
        output.push(Vec::new());

        for num in line {
            if *num == None || (num.unwrap() + 1) as usize > letvec.len() {
                output[y].push(None);
            } else {
                output[y].push(Some(letvec[num.unwrap() as usize]));
            }
        }
    }
    return output;
}