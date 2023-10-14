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

pub fn push_ascii(text: &str, stream: &mut Badstream) {
    for i in text.chars() {
        if i.is_ascii() {
            let a = i as u8;
            for k in (0..=7).rev() {
                stream.push((a & (1 << k)) != 0);
            }
        }
    }
}

// ref. pg. 34
// 0xEC and 0x11 are the pad codewords, 11101100 and 00010001
pub fn pad_to(codeword_length: usize, stream: &mut Badstream) {
    if stream.len() % 8 != 0 {
        stream.resize(stream.len() + (8 - stream.len() % 8), false);
    }
    for i in 0..(codeword_length - (stream.len() / 8)) {
        let a = [0xEC, 0x11][i % 2];
        for k in (0..=7).rev() {
            stream.push((a & (1 << k)) != 0);
        }
    }
}

pub fn push_codewords(codewords: &[u8], stream: &mut Badstream) {
    // padding
    if stream.len() % 8 != 0 {
        stream.resize(stream.len() + (8 - stream.len() % 8), false);
    }
    for &a in codewords {
        for k in (0..=7).rev() {
            stream.push((a & (1 << k)) != 0);
        }
    }
}

pub fn push_bits(bits: &str, stream: &mut Badstream) {
    for number in bits.chars() {
        stream.push(match number {
            '0' => false,
            '1' => true,
            _ => panic!(),
        });
    }
}

pub fn write_badstream_to_bitmap<T: QR>(stream: &Badstream, bitmap: &mut T) {
    let version = bitmap.qr_version().unwrap();
    let max = version_to_max_index(version);
    let (mut x, mut y) = (max, max);
    for (a, &i) in stream.iter().enumerate() {
        bitmap.set_bit(x, y, i);
        if let Some((x2, y2)) = next_data_bit(x, y, version) {
            (x, y) = (x2, y2);
        } else {
            assert_eq!(a + 1, stream.len());
            break;
        }
    }
}
