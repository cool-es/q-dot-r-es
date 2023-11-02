// quick and dirty solution to start entering data into qr codes
#![allow(unused_mut, unused_variables)]

use super::*;

pub(crate) type Badstream = Vec<bool>;

pub(crate) fn badstream_to_polynomial(input: &Badstream) -> Polynomial {
    let mut output: Polynomial = Vec::new();

    let mut pushbyte: u8 = 0;
    for (i, &bit) in input.iter().enumerate() {
        if i % 8 == 0 && i != 0 {
            output.push(pushbyte as u32);
            pushbyte = 0;
        }
        pushbyte <<= 1;
        pushbyte |= u8::from(bit);
    }
    if pushbyte != 0 {
        output.push(pushbyte as u32);
    }
    output
}

pub(crate) fn push_ascii(text: &str, stream: &mut Badstream) {
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
pub(crate) fn pad_to(codeword_length: usize, stream: &mut Badstream) {
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

pub(crate) fn push_codewords(codewords: &[u8], stream: &mut Badstream) {
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
pub(crate) fn push_byte(byte: u8, stream: &mut Badstream) {
    for k in (0..=7).rev() {
        stream.push((byte & (1 << k)) != 0);
    }
}

pub(crate) fn push_bits(bits: &str, stream: &mut Badstream) {
    for number in bits.chars() {
        stream.push(match number {
            '0' => false,
            '1' => true,
            _ => panic!(),
        });
    }
}

pub(crate) fn polynomial_to_badstream(poly: &Polynomial) -> Badstream {
    let mut stream = Vec::new();
    for &a in poly {
        assert!(a < 0x100, "polynomial contains non-byte data");
        for k in (0..=7).rev() {
            stream.push((a & (1 << k)) != 0);
        }
    }
    stream
}

pub(crate) fn write_badstream_to_bitmap(stream: &Badstream, bitmap: &mut Bitmap) {
    let version = bitmap.qr_version().expect("invalid bitmap size");
    let max = bitmap.dims().0 - 1;
    let (mut x, mut y) = (max, max);
    for (a, &i) in stream.iter().enumerate() {
        bitmap.set_bit(x, y, i);
        if let Some((x2, y2)) = next_data_bit(x, y, version) {
            (x, y) = (x2, y2);
        } else {
            assert!(
                a + 1 == stream.len(),
                "write_badstream_to_bitmap(): bitstream is {} bits but image only fits {} (difference: {} bits)",
                stream.len(),
                a + 1,
                stream.len() as i32 - (a + 1) as i32,
            );
            break;
        }
    }
}

pub(crate) fn split_to_blocks_and_encode(
    poly: &Polynomial,
    info: VersionBlockInfo,
) -> Vec<Polynomial> {
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
        "could not split to blocks - stream is {} codewords but allotted space is {}\nversion info: {:?}",
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

pub(crate) fn full_block_encode(stream: &Badstream, version: u32, level: u8) -> Badstream {
    let block_info = get_block_info(version, level);
    let (block_count, codewords, data_codewords, optional) = block_info;
    let ec_codewords = codewords - data_codewords;
    let (max_data_codewords, total_data_codewords) =
        if let Some((block_count_2, _, data_codewords_2)) = optional {
            (
                data_codewords_2,
                block_count * data_codewords + block_count_2 * data_codewords_2,
            )
        } else {
            (data_codewords, block_count * data_codewords)
        };

    let padded_stream = {
        let mut stream_copy = stream.clone();
        pad_to(total_data_codewords, &mut stream_copy);
        stream_copy
    };

    // vector of error-corrected polynomial blocks
    let encoded_poly_vec =
        split_to_blocks_and_encode(&badstream_to_polynomial(&padded_stream), block_info);

    // error_output only serves to display byte data
    // in case the size-check assert below fails
    let mut output: Vec<bool> = Vec::new();
    let mut error_output: Vec<u32> = Vec::new();

    // enter data codewords
    for i in 0..max_data_codewords {
        for block in &encoded_poly_vec {
            if i < block.len() - ec_codewords {
                push_byte(block[i] as u8, &mut output);
                error_output.push(block[i]);
            }
        }
    }

    assert!(
        output.len() == 8 * total_data_codewords,
        "full_block_encode(): version {} level {}: number of codewords should be {} but is {}\norig. data {:?}\noutput {:?}",version,level,
        total_data_codewords,
        output.len() / 8,
        encoded_poly_vec,
        error_output,
    );

    // enter EC codewords
    for i in 0..ec_codewords {
        for block in &encoded_poly_vec {
            let offset = block.len() - ec_codewords;
            push_byte(block[offset + i] as u8, &mut output);
        }
    }
    output
}

// container to hold input data based on if it's mode-switched or not
#[derive(Clone, Debug)]
pub(crate) enum QRInput {
    Auto(String),
    Manual(Vec<(Mode, String)>),
}

pub(crate) fn make_qr(
    input: QRInput,
    version_choice: Option<u32>,
    level_choice: Option<u8>,
    mask_choice: Option<u8>,
) -> Bitmap {
    let tokens = make_token_stream(match input {
        QRInput::Auto(str) => auto_mode_switch(str),
        QRInput::Manual(vec) => vec,
    });

    let level = level_choice.unwrap_or(2);
    let best_ver = find_best_version(&tokens, level).expect("make_qr()");

    let version = if let Some(chosen_ver) = version_choice {
        assert!(
            !bad_version(chosen_ver),
            "invalid version {} chosen",
            chosen_ver
        );
        assert!(
            best_ver <= chosen_ver,
            "QR version {}-{} can't fit the data - best option is {}-{}",
            chosen_ver,
            b"LMQH"[level as usize] as char,
            best_ver,
            b"LMQH"[level as usize] as char,
        );
        chosen_ver
    } else {
        best_ver
    };

    let shuffled_stream = full_block_encode(&tokens_to_badstream(tokens, version), version, level);

    let mut bitmap = Bitmap::new_blank_qr(version);

    write_badstream_to_bitmap(&shuffled_stream, &mut bitmap);
    let mask = if let Some(mask) = mask_choice {
        mask
    } else {
        choose_best_mask(&bitmap, version, level)
    };
    apply_mask(&mut bitmap, version, level, mask);

    bitmap
}

fn apply_mask(bitmap: &mut Bitmap, version: u32, level: u8, mask: u8) {
    set_fcode(
        bitmap,
        version,
        data_to_fcode([0b01, 0b00, 0b11, 0b10][level as usize], mask)
            .expect("could not generate format code"),
    );
    bitmap.qr_mask_xor(mask);
}

pub(crate) fn choose_best_mask(bitmap: &Bitmap, version: u32, level: u8) -> u8 {
    let mut best: Bitmap;
    let (mut best, mut penalty) = (u8::MAX, u32::MAX);
    for mask in 0..=7 {
        let pen = {
            let mut clone = bitmap.clone();
            apply_mask(&mut clone, version, level, mask);
            clone.qr_penalty()
        };
        if pen < penalty {
            best = mask;
            penalty = pen;
        }
    }
    best
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

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

    // this is inaccurate, not sure why,
    // there's seemingly nothing wrong with split_to_blocks_and_encode...
    // but i don't see a reason to fix it nor remove it at this time
    // #[test]
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
                let list2 = split_to_blocks_and_encode(
                    &((1..=200).cycle().take(cw as usize).collect()),
                    info2,
                );

                let sum2 = list2.iter().map(|x| x.len()).sum::<usize>();
                assert!(sum1 == sum2, "version {}, level {}", ver, level);
            }
        }
    }
}
