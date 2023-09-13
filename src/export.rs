//adapt to my needs!!

/*
use bmp::{px, Image, Pixel};
use std::env;

const SIZE: u32 = 256;

fn main() {
    let mut arg = Vec::new();
    arg = env::args().collect();

    if (arg.len() != 2) || (arg[1].len() != 6) {
        panic!("Bad input >:(");
    }

    let r = u32::from_str_radix(&arg[1][0..2], 16).expect("No no no!!");
    let g = u32::from_str_radix(&arg[1][2..4], 16).expect("No no no!!");
    let b = u32::from_str_radix(&arg[1][4..6], 16).expect("No no no!!");

    println!("{:?}", (r, g, b));

    // i took the below code from the bmp
    // crate's code example.... sorry.

    let mut img = Image::new(SIZE, SIZE);

    for (x, y) in img.coordinates() {
        img.set_pixel(x, y, px!(r, g, b));
    }

    let _ = img.save("img.bmp");
}
*/

pub fn print_pattern (pattern: &Vec<Vec<bool>>) {
    for line in pattern {
        // this routine is made to print 3x2 rectangles of characters
        // which makes a square in my 8x12px terminal font
        let mut row: String = String::new();
        for element in line {
            if *element {
                row.push_str("░░░");
            } else {
                row.push_str("███");
            }
        }
        println!("{row}");
        println!("{row}");
    }
}