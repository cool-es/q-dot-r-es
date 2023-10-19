// quick and dirty solution to start entering data into qr codes
// #![allow(unused_mut, unused_variables)]

const HELLOMSG: [u8; 8] = [0x40, 0x66, 0x86, 0x56, 0xC6, 0xC6, 0xF2, 0x10];

use super::*;

pub type Badstream = Vec<bool>;

pub fn badstream_to_polynomial(input: &Badstream) -> Polynomial {
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
            // for k in (0..=7).rev() {
            //     stream.push((a & (1 << k)) != 0);
            // }
            push_byte(a, stream);
        }
    }
}

// ref. pg. 34
// 0xEC and 0x11 are the pad codewords, 11101100 and 00010001
pub fn pad_to(codeword_length: usize, stream: &mut Badstream) {
    // stream is too long
    if stream.len().div_ceil(8) > codeword_length {
        panic!(
            "pad_to() out of bounds: stream is {} bits but bound is {}",
            stream.len(),
            8 * codeword_length
        )
    } else if stream.len().div_ceil(8) > codeword_length - 1 && stream.len() % 8 != 0 {
        panic!(
            "pad_to() error: {} bits away from bound; code will not scan",
            codeword_length * 8 - stream.len()
        )
    }

    // pad to next codeword boundary with zeros
    if stream.len() % 8 != 0 {
        stream.resize(stream.len().next_multiple_of(8), false);
    }

    // for i in 0..(codeword_length
    //     .checked_sub(stream.len() / 8)
    //     .unwrap_or_default())
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
    for &byte in codewords {
        // for k in (0..=7).rev() {
        //     stream.push((byte & (1 << k)) != 0);
        // }
        push_byte(byte, stream);
    }
}

// pushes a byte without any alignment checks
pub fn push_byte(byte: u8, stream: &mut Badstream) {
    for k in (0..=7).rev() {
        stream.push((byte & (1 << k)) != 0);
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

pub fn polynomial_to_badstream(poly: &Polynomial) -> Badstream {
    let mut stream = Vec::new();
    for &a in poly {
        assert!(a < 0x100, "polynomial contains non-byte data");
        for k in (0..=7).rev() {
            stream.push((a & (1 << k)) != 0);
        }
    }
    stream
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

pub fn split_to_blocks_and_encode(poly: &Polynomial, info: VersionBlockInfo) -> Vec<Polynomial> {
    // number of blocks of this type, codewords per block, data codewords per block
    // note that the number of error correcting codewords is the same for all blocks!
    let (bc, cw, dcw, optional) = info;
    let (bc2, _, dcw2) = optional.unwrap_or((0, 0, 0));

    // if optional.is_some() {
    //     panic!("multiple block types are not supported yet")
    // }
    // let (bc2, dcw2) = (0, 0);

    // check to make sure poly will split evenly
    assert!(
        poly.len() == dcw * bc + dcw2 * bc2,
        "could not split to blocks - stream is {} codewords but alotted space is {}",
        poly.len(),
        dcw * bc + dcw2 * bc2
    );

    let mut unencoded: Vec<Polynomial> = Vec::new();

    let (first, second) = poly.split_at(bc * dcw);

    for i in 0..bc {
        let (a, b) = (i * dcw, (i + 1) * dcw);
        // unencoded.push(poly[a..b].to_vec());
        unencoded.push(first[a..b].to_vec());
    }
    for i in 0..bc2 {
        let (a, b) = (i * dcw2, (i + 1) * dcw2);
        // unencoded.push(poly[a..b].to_vec());
        unencoded.push(second[a..b].to_vec());
    }

    let mut output = Vec::new();

    for i in unencoded {
        output.push(encode_message(&i, (cw - dcw) as u32));
    }

    output
}

pub fn full_block_encode(stream: &Badstream, version: u32, level: u8) -> Badstream {
    let info = get_block_info(version, level);
    let (bc, cw, dcw, opt) = info;
    let ec_cw = cw - dcw;
    let max_dcw = if let Some((_, _, dcw2)) = opt {
        dcw2
    } else {
        dcw
    };
    let data_codewords = if let Some((bc2, _, dcw2)) = opt {
        bc * dcw + bc2 * dcw2
    } else {
        bc * dcw
    };

    let padded_stream = {
        let mut stream_copy = stream.clone();
        pad_to(data_codewords as usize, &mut stream_copy);
        stream_copy
    };

    let poly = badstream_to_polynomial(&padded_stream);
    let polys = split_to_blocks_and_encode(&poly, info);

    let mut output = Vec::new();

    // enter data codewords
    for i in 0..max_dcw {
        for block in &polys {
            if i < block.len() {
                push_byte(block[i] as u8, &mut output);
            }
        }
    }

    // enter EC codewords
    for i in 0..ec_cw {
        for block in &polys {
            let offset = block.len() - 1 - ec_cw;
            push_byte(block[offset + i] as u8, &mut output);
        }
    }

    // assert!(output.len() == )

    output
}

pub fn generate_qr_code(mode_data: &[(u8, &str)], version: u32, level: u8) -> ImgRowAligned {
    let stream: &mut Badstream = &mut invoke_modes(mode_data, version);

    let shuffled_stream = full_block_encode(stream, version, level);

    let mut best = (0, 0);
    let mut variants = Vec::new();
    for mask in 0..=7 {
        {
            let mut bitmap = ImgRowAligned::new_blank_qr(version);

            set_fcode(
                &mut bitmap,
                version,
                (0, 0),
                data_to_fcode([0b01, 0b00, 0b11, 0b10][level as usize], mask).unwrap(),
            );
            write_badstream_to_bitmap(&shuffled_stream, &mut bitmap);
            bitmap.qr_mask_xor(mask);

            if bitmap.qr_penalty() > best.0 {
                best.0 = bitmap.qr_penalty();
                best.1 = mask as usize;
            }

            variants.push(bitmap);
        }
    }
    variants[best.1].clone()
}
