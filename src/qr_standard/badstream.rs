use super::{
    bitstream::{self, search, Mode},
    image, tables,
};
use crate::rdsm::{galois, poly};

// a bitstream with one bit per u8
// bit is 'true' iff the u8 != 0
pub type Badstream = Vec<u8>;

pub fn badstream_to_polynomial(input: &Badstream) -> poly::Polynomial {
    let mut output: poly::Polynomial = Vec::new();

    let mut pushbyte: u8 = 0;
    for (i, &bit) in input.iter().enumerate() {
        if i % 8 == 0 && i != 0 {
            output.push(pushbyte as galois::Element);
            pushbyte = 0;
        }
        pushbyte <<= 1;
        pushbyte |= bit;
    }
    output.push(pushbyte as galois::Element);
    output
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
        stream.resize(stream.len().next_multiple_of(8), 0);
    }

    // for i in 0..(codeword_length
    //     .checked_sub(stream.len() / 8)
    //     .unwrap_or_default())
    for i in 0..(codeword_length - (stream.len() / 8)) {
        let a = [0xEC, 0x11][i % 2];
        for k in (0..=7).rev() {
            stream.push(a & (1 << k));
        }
    }
}

/// pushes a byte without any alignment checks
pub fn push_byte(byte: u8, stream: &mut Badstream) {
    for k in (0..=7).rev() {
        stream.push(byte & (1 << k));
    }
}

pub fn push_bits(bits: &str, stream: &mut Badstream) {
    for number in bits.chars() {
        stream.push(match number {
            '0' => 0,
            '1' => 1,
            _ => panic!(),
        });
    }
}

pub fn write_badstream_to_bitmap(stream: &Badstream, bitmap: &mut image::Bitmap) {
    let version = bitmap.qr_version().expect("invalid bitmap size");
    let max = bitmap.dims().0 - 1;
    let (mut x, mut y) = (max, max);
    for (a, &i) in stream.iter().enumerate() {
        bitmap.set_bit(x, y, i != 0);
        match super::next_data_bit(x, y, version) {
            Some((x2, y2)) => {
                (x, y) = (x2, y2);
            }
            None => {
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
}

pub fn split_to_blocks_and_encode(
    poly: &poly::Polynomial,
    info: tables::VersionBlockInfo,
) -> Vec<poly::Polynomial> {
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

    let mut unencoded: Vec<poly::Polynomial> = Vec::new();

    let (first, second): (&[galois::Element], &[galois::Element]) = if optional.is_some() {
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
        output.push(poly::encode_message(&i, (cw - dcw) as u32));
    }

    output
}

pub fn full_block_encode(stream: &Badstream, version: u32, level: u8) -> Badstream {
    let block_info = tables::get_block_info(version, level);
    let (block_count, codewords, data_codewords, optional) = block_info;
    let ec_codewords = codewords - data_codewords;
    let (max_data_codewords, total_data_codewords) = match optional {
        Some((block_count_2, _, data_codewords_2)) => (
            data_codewords_2,
            block_count * data_codewords + block_count_2 * data_codewords_2,
        ),
        None => (data_codewords, block_count * data_codewords),
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
    let mut output: Badstream = Vec::new();
    let mut error_output: Vec<galois::Element> = Vec::new();

    // enter data codewords
    for i in 0..max_data_codewords {
        for block in &encoded_poly_vec {
            if i < block.len() - ec_codewords {
                push_byte(block[i], &mut output);
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
            push_byte(block[offset + i], &mut output);
        }
    }
    output
}

/// container to hold input data based on if it's mode-switched or not
#[derive(Clone, Debug)]
pub enum QRInput {
    Auto(String),
    Manual(Vec<(Mode, String)>),
}

pub fn make_qr(
    input: QRInput,
    version_choice: Option<u32>,
    level_choice: Option<u8>,
    mask_choice: Option<u8>,
) -> image::Bitmap {
    let level = level_choice.unwrap_or(2);

    // is utf8 (unicode) encoding necessary?
    let utf8_encoding = match input {
        // auto: check if string contains non-ascii chars
        QRInput::Auto(ref str) => !str.is_ascii(),

        // manual: check if any ASCII segment contains non-ascii chars
        QRInput::Manual(ref vec) => vec.iter().any(|(m, s)| *m == Mode::ASCII && !s.is_ascii()),
    };

    let input = match input {
        QRInput::Auto(str) => find_best_mode_optimization(str, level),
        QRInput::Manual(vec) => vec,
    };

    // back up mode vector to info structure
    #[cfg(feature = "demo")]
    crate::demo::ops::set_modes(&input);

    let tokens = bitstream::make_token_stream(
        input,
        if utf8_encoding {
            Some(tables::eci::UTF8)
        } else {
            None
        },
    );

    let best_ver = bitstream::find_best_version(&tokens, level).expect("make_qr()");

    let version = match version_choice {
        Some(chosen_ver) => {
            assert!(
                !super::bad_version(chosen_ver),
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
        }
        None => best_ver,
    };

    let shuffled_stream = full_block_encode(
        &bitstream::tokens_to_badstream(tokens, version),
        version,
        level,
    );

    let mut bitmap = image::Bitmap::new_blank_qr(version);

    write_badstream_to_bitmap(&shuffled_stream, &mut bitmap);
    match mask_choice {
        Some(mask) => {
            apply_mask(&mut bitmap, version, level, mask);
        }
        None => {
            apply_best_mask(&mut bitmap, version, level);
        }
    }

    bitmap
}

fn apply_mask(bitmap: &mut image::Bitmap, version: u32, level: u8, mask: u8) {
    super::set_fcode(
        bitmap,
        version,
        super::data_to_fcode([0b01, 0b00, 0b11, 0b10][level as usize], mask)
            .expect("could not generate format code"),
    );
    bitmap.qr_mask_xor(mask);
}

pub fn apply_best_mask(bitmap: &mut image::Bitmap, version: u32, level: u8) {
    let mut best = image::Bitmap::new(1, 1);
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
    // calculated once to not waste processing time
    let maybe_eci_header = if !str.is_ascii() { 8 } else { 0 };

    // the limiting sizes for each code class, in codewords
    let class_limits = {
        let dcw = tables::DATA_CODEWORDS[level as usize];
        [dcw[10 - 1], dcw[27 - 1]]
    };

    // check if the code fits in the first class, (version 1..)
    // then the second class (version 10..)
    for (class, limit) in class_limits.iter().enumerate() {
        let marked_string = search::optimize_mode(&str, class as u8);

        // calculate the total message size, in bits
        let cost = marked_string
            .iter()
            .map(|(mode, string)| bit_cost(string.len(), class, *mode))
            .sum::<usize>()
            + maybe_eci_header;

        // if the message fits the limit with at least one codeword,
        // or exactly 0 bits, to spare, then return it
        if let Some(diff) = (8 * limit).checked_sub(cost) {
            if diff > 7 || diff == 0 {
                return marked_string;
            }
        }
    }

    // code must be third class (version 27..),
    // so no calculation is necessary
    search::optimize_mode(&str, 2)
}

// verified accurate
// returns the number of bits it takes to print `count` characters
// in a given mode and size class of qr code
fn bit_cost(count: usize, class: usize, mode: Mode) -> usize {
    let cc_bits = tables::CC_INDICATOR_BITS[class];
    4 + match mode {
        Mode::Numeric => 4 + cc_bits[0] + ((10 * count + 1) as f32 / 3.0).round() as usize,
        Mode::AlphaNum => 4 + cc_bits[1] + 11 * (count / 2) + 6 * (count % 2),
        Mode::ASCII => 4 + cc_bits[2] + 8 * count,
    }
}
