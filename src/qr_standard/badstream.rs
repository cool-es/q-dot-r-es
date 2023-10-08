// quick and dirty solution to start entering data into qr codes
#![allow(unused_mut, unused_variables)]

use super::*;

pub type Badstream = Vec<bool>;

pub fn badstream_to_poly(input: &Badstream) -> Polynomial {
    let mut output: Polynomial = Vec::new();

    let mut pushbyte = 0;
    for i in 0..input.len() {
        if i % 8 == 0 && i != 0 {
            output.push(pushbyte);
            pushbyte = 0;
        }
        pushbyte <<= 1;
        pushbyte |= Element::from(input[i]);
    }
    if pushbyte != 0 {
        output.push(pushbyte);
    }

    output
}

pub fn ascii_to_badstream(input: &str) -> Badstream {
    let mut output = Badstream::new();
    for i in input.chars() {
        if i.is_ascii() {
            let a = i as u8;
            for k in (0..=7).rev() {
                output.push((a & (1 << k)) != 0);
            }
        }
    }
    output
}
