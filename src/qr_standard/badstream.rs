// quick and dirty solution to start entering data into qr codes
#![allow(unused_mut, unused_variables)]

const HELLOMSG: [u8; 8] = [0x40, 0x66, 0x86, 0x56, 0xC6, 0xC6, 0xF2, 0x10];

use super::*;

pub type Badstream = Vec<bool>;

pub fn badstream_to_poly(input: &Badstream) -> Polynomial {
    let mut output: Polynomial = Vec::new();

    let mut pushbyte: u8 = 0;
    for i in 0..input.len() {
        if i % 8 == 0 && i != 0 {
            output.push(pushbyte as u32);
            pushbyte = 0;
        }
        pushbyte <<= 1;
        pushbyte |= u8::from(input[i]);
    }
    if pushbyte != 0 {
        output.push(pushbyte as u32);
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

pub fn pad_to(codewords: usize, stream: &mut Badstream) {
    if stream.len() % 8 != 0 {
        stream.resize(stream.len() + (8 - stream.len() % 8), false);
    }
    for i in 0..(codewords - (stream.len() / 8)) {
        let a = [0xEC, 0x11][i % 2];
        for k in (0..=7).rev() {
            stream.push((a & (1 << k)) != 0);
        }
    }
}

pub fn push_bytes(bytes: &[u8], stream: &mut Badstream) {
    for &a in bytes {
        for k in (0..=7).rev() {
            stream.push((a & (1 << k)) != 0);
        }
    }
}

// incorrect - refer to pg. 26
// pub fn add_numeric_to_badstream(nums: u32, stream: &mut Badstream) {
//     let size = {
//         if let Some(k) = nums.checked_ilog10() {
//             k + 1
//         } else {
//             0
//         }
//     };

//     if size % 3 != 0 {
//         // ???
//         todo!()
//     }

//     for i in (1..(size / 3)).rev() {
//         let a = (nums / (1000 * i)) as usize;
//         for k in (0..=10).rev() {
//             stream.push((a & (1 << k)) != 0);
//         }
//     }
// }
