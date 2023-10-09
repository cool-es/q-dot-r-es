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

pub fn add_ascii_to_badstream(text: &str, stream: &mut Badstream) {
    for i in text.chars() {
        if i.is_ascii() {
            let a = i as u8;
            for k in (0..=7).rev() {
                stream.push((a & (1 << k)) != 0);
            }
        }
    }
}

// incorrect - refer to pg. 26
pub fn add_numeric_to_badstream(nums: u32, stream: &mut Badstream) {
    let size = {
        if let Some(k) = nums.checked_ilog10() {
            k + 1
        } else {
            0
        }
    };

    if size % 3 != 0 {
        // ???
        todo!()
    }

    for i in (1..(size / 3)).rev() {
        let a = (nums / (1000 * i)) as usize;
        for k in (0..=10).rev() {
            stream.push((a & (1 << k)) != 0);
        }
    }
}
