use super::*;

pub(crate) type Badstream = Vec<bool>;

pub(crate) fn badstream_to_polynomial(input: &Badstream) -> Polynomial {
    let mut output: Polynomial = Vec::new();

    let mut pushbyte: u8 = 0;
    for (i, &bit) in input.iter().enumerate() {
        if i % 8 == 0 && i != 0 {
            output.push(pushbyte as Element);
            pushbyte = 0;
        }
        pushbyte <<= 1;
        pushbyte |= u8::from(bit);
    }
    if pushbyte != 0 {
        output.push(pushbyte as Element);
    }
    output
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

/// pushes a byte without any alignment checks
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

    // check to make sure poly will split evenly
    assert!(
        poly.len() == dcw * bc + dcw2 * bc2,
        "could not split to blocks - stream is {} codewords but allotted space is {}\nversion info: {:?}",
        poly.len(),
        dcw * bc + dcw2 * bc2,
        info
    );

    let mut unencoded: Vec<Polynomial> = Vec::new();

    let (first, second): (&[Element], &[Element]) = if optional.is_some() {
        poly.split_at(bc * dcw)
    } else {
        (poly.as_slice(), &[])
    };

    for i in 0..bc {
        let (a, b) = (i * dcw, (i + 1) * dcw);
        unencoded.push(first[a..b].to_vec());
    }
    for i in 0..bc2 {
        let (a, b) = (i * dcw2, (i + 1) * dcw2);
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
    let mut error_output: Vec<Element> = Vec::new();

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

/// container to hold input data based on if it's mode-switched or not
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
    let level = level_choice.unwrap_or(2);
    let mut utf8_encoding = false;

    let input = match input {
        QRInput::Auto(str) => {
            utf8_encoding = !str.is_ascii();
            find_best_mode_optimization(str, level)
        }
        QRInput::Manual(vec) => {
            for i in vec.iter().filter(|(m, _)| *m == ASCII) {
                if !i.1.is_ascii() {
                    utf8_encoding = true;
                    break;
                }
            }
            vec
        }
    };

    let tokens = if utf8_encoding {
        let mut vec = vec![Token::EciChange(eci::UTF8)];
        vec.append(&mut make_token_stream(input));
        vec
    } else {
        make_token_stream(input)
    };

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
    if let Some(mask) = mask_choice {
        apply_mask(&mut bitmap, version, level, mask);
    } else {
        apply_best_mask(&mut bitmap, version, level);
    }

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

pub(crate) fn apply_best_mask(bitmap: &mut Bitmap, version: u32, level: u8) {
    let mut best = Bitmap::new(1, 1);
    let mut penalty = u32::MAX;
    for mask in 0..=7 {
        let mut clone = bitmap.clone();
        apply_mask(&mut clone, version, level, mask);
        let pen = clone.qr_penalty();

        if pen < penalty {
            best = clone;
            penalty = pen;
        }
    }
    *bitmap = best;
}

/// Find the best possible mode optimization for the message.
///
/// Since mode optimization is defined circularly (the mode
/// switching influences the message size, which influences
/// what version QR code is chosen, which influences what
/// mode switching is optimal), it's necessary to do this
/// step before the optimal QR version can be decided on.
fn find_best_mode_optimization(str: String, level: u8) -> Vec<(Mode, String)> {
    use super::bitstream::search::optimize_mode;

    let maybe_eci_header = if !str.is_ascii() { 8 } else { 0 };

    // the limiting sizes for each code class, in codewords
    // = [274, 1468]
    let class_limits = {
        let dcw = DATA_CODEWORDS[level as usize];
        [dcw[10 - 1], dcw[27 - 1]]
    };

    // check if the code fits in the first class, (version 1..)
    // then the second class (version 10..)
    for class in 0..2 {
        let marked_string = optimize_mode(&str, class as u8);

        // calculate the total message size, in bits
        let cost = marked_string
            .iter()
            .map(|(mode, string)| bit_cost(string.len(), class, *mode))
            .sum::<usize>()
            + maybe_eci_header;

        // if the message fits the limit with at least one codeword,
        // or exactly 0 bits, to spare, then return it
        if let Some(diff) = (8 * class_limits[class]).checked_sub(cost) {
            if diff > 7 || diff == 0 {
                return marked_string;
            }
        }
    }

    // code must be third class (version 27..),
    // so no calculation is necessary
    optimize_mode(&str, 2)
}

// verified accurate
// returns the number of bits it takes to print `count` characters
// in a given mode and size class of qr code
fn bit_cost(count: usize, class: usize, mode: Mode) -> usize {
    let cc_bits = CC_INDICATOR_BITS[class];
    4 + match mode {
        Numeric => 4 + cc_bits[0] + ((10 * count + 1) as f32 / 3.0).round() as usize,
        AlphaNum => 4 + cc_bits[1] + 11 * (count / 2) + 6 * (count % 2),
        ASCII => 4 + cc_bits[2] + 8 * count,
        Kanji => todo!("refer to kanji bit information"),
    }
}
