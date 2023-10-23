// quick and dirty solution to start entering data into qr codes
#![allow(unused_mut, unused_variables)]

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
            assert!(
                a + 1 == stream.len(),
                "🚨 write_badstream_to_bitmap(): bitstream is {} bits but image only fits {} (difference: {} bits)",
                stream.len(),
                a + 1,
                stream.len() as i32 - (a + 1) as i32,
            );
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
        "could not split to blocks - stream is {} codewords but alotted space is {}\nversion info: {:?}",
        poly.len(),
        dcw * bc + dcw2 * bc2,
        info
    );

    let mut unencoded: Vec<Polynomial> = Vec::new();

    let (first, second): (&[u32], &[u32]) = if optional.is_some() {
        poly.split_at(bc * dcw)
    } else {
        (poly.as_slice(), &[])
    };

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
    let total_dcw = if let Some((bc2, _, dcw2)) = opt {
        bc * dcw + bc2 * dcw2
    } else {
        bc * dcw
    };

    let padded_stream = {
        let mut stream_copy = stream.clone();
        pad_to(total_dcw as usize, &mut stream_copy);
        stream_copy
    };

    let poly = badstream_to_polynomial(&padded_stream);
    let polys = split_to_blocks_and_encode(&poly, info);

    let mut output = Vec::new();
    let mut dummy_out = Vec::new();

    // enter data codewords
    for i in 0..max_dcw {
        for block in &polys {
            if i < block.len() - ec_cw {
                push_byte(block[i] as u8, &mut output);
                dummy_out.push(block[i]);
            }
        }
    }

    assert!(
        output.len() == 8 * total_dcw,
        "full_block_encode(): version {} level {}: number of codewords should be {} but is {}\norig. data {:?}\noutput {:?}",version,level,
        total_dcw,
        output.len() / 8,
        polys,
        dummy_out,
    );

    // enter EC codewords
    for i in 0..ec_cw {
        for block in &polys {
            let offset = block.len() - ec_cw;
            push_byte(block[offset + i] as u8, &mut output);
        }
    }

    // assert!(output.len() == )

    output
}

#[test]
fn block_encode_is_well_behaved() {
    let poly: Polynomial = (1..=19).collect();
    let stream = polynomial_to_badstream(&poly);
    let enc_poly = encode_message(&poly, 7);
    let enc_stream = full_block_encode(&stream, 1, 0);
    let polynomialized_stream = badstream_to_polynomial(&enc_stream);

    assert!(
        polynomialized_stream.len() == enc_poly.len(),
        "mismatched lengths - is {}, should be {}",
        polynomialized_stream.len(),
        enc_poly.len(),
    );

    for i in 0..polynomialized_stream.len() {
        assert!(
            polynomialized_stream[i] == enc_poly[i],
            "mismatch at index {} - byte is {:#04X}, should be {:#04X}\nbad  {:?}\ngood {:?}",
            i,
            polynomialized_stream[i],
            enc_poly[i],
            polynomialized_stream,
            enc_poly,
        );
    }
}

pub(crate) fn make_qr(
    mode_data: Vec<(Mode, String)>,
    version_choice: Option<u32>,
    level: u8,
    mask_choice: Option<u8>,
) -> ImgRowAligned {
    let tokens = make_token_stream(mode_data);

    let version = if let Some(choice) = version_choice {
        choice
    } else {
        find_best_version(&tokens, level)
    };

    let stream: &mut Badstream = &mut tokens_to_badstream(tokens, version);
    let shuffled_stream = full_block_encode(stream, version, level);
    let mut bitmap = ImgRowAligned::new_blank_qr(version);

    write_badstream_to_bitmap(&shuffled_stream, &mut bitmap);
    let mask = if let Some(mask) = mask_choice {
        mask
    } else {
        choose_best_mask(&bitmap)
    };
    apply_mask(&mut bitmap, version, level, mask);

    bitmap
}

fn apply_mask(bitmap: &mut ImgRowAligned, version: u32, level: u8, mask: u8) {
    set_fcode(
        bitmap,
        version,
        (0, 0),
        data_to_fcode([0b01, 0b00, 0b11, 0b10][level as usize], mask).unwrap(),
    );
    bitmap.qr_mask_xor(mask);
}

pub fn choose_best_mask(bitmap: &ImgRowAligned) -> u8 {
    if bitmap.qr_version() > Some(27) {
        eprintln!(
            "choose_best_mask(): qr code too large for penalty routine, defaulting to mask 3"
        );
        return 3;
    }
    let mut best: ImgRowAligned;
    let (mut best, mut penalty) = (u8::MAX, u32::MAX);
    for mask in 0..=7 {
        let pen = {
            let mut clone = bitmap.clone();
            clone.qr_mask_xor(mask);
            clone.qr_penalty()
        };
        if pen < penalty {
            best = mask;
            penalty = pen;
        }
    }
    best
}

#[test]
fn block_encode_is_consistent() {
    for version in 1..=40 {
        let (bc, cw, dcw, opt) = get_block_info(version, 3);
        let data_limit = if let Some((bc2, cw2, dcw2)) = opt {
            bc * dcw + bc2 + dcw2
        } else {
            bc * dcw
        };

        let block = |level| {
            full_block_encode(
                &polynomial_to_badstream(&((100..200).cycle().take(data_limit - 3).collect())),
                version,
                level,
            )
            .len()
        };
        let (l, m, q, h) = (block(0), block(1), block(2), block(3));
        assert!(
            l == m && l == q && l == h,
            "version {}, bit length inconsistent:\nl: {}\nm: {}\nq: {}\nh: {}",
            version,
            l,
            m,
            q,
            h
        );
    }
}

#[test]
fn split_to_blocks_is_consistent() {
    for ver in 1..=40 {
        let info = get_block_info(ver, 0);
        let cw = if let Some((a, _, b)) = info.3 {
            a * b + info.0 * info.2
        } else {
            info.0 * info.2
        } as u32;
        let list1 =
            split_to_blocks_and_encode(&((1..=200).cycle().take(cw as usize).collect()), info);
        let sum1 = list1.iter().map(|x| x.len()).sum::<usize>();

        for level in 1..=3 {
            let info2 = get_block_info(ver, level);
            let cw2 = if let Some((a, _, b)) = info2.3 {
                a * b + info2.0 * info2.2
            } else {
                info2.0 * info2.2
            } as u32;
            let list2 =
                split_to_blocks_and_encode(&((1..=200).cycle().take(cw as usize).collect()), info2);

            let sum2 = list2.iter().map(|x| x.len()).sum::<usize>();
            assert!(sum1 == sum2, "version {}, level {}", ver, level);
        }
    }
}
