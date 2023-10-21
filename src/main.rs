// work in progress, suppressing warnings
#![allow(dead_code)]

mod image;
mod qr_standard;
mod rdsm;

use qr_standard::Mode::*;

use image::*;
use qr_standard::*;
use rdsm::*;
// use testutil::*;

fn main() {
    let _binding = String::from("1234567890".chars().cycle().take(1337).collect::<String>());
    let mode_data = &[
        // (0, _binding.as_str()),
        //
        // (1, "PUBLIC SERVICE ANNOUNCEMENT - "),
        (ASCII,"hello!! so it seems as though my block division routine works, and i am now able to make \"fuck off\"-size qr codes with way way way too much data in them. i remember it as if it was only last week i only had the ability to say a few words in my qr code... such a tragic fate. but now i can just go on and on and on and bore everyone to death because this qr code just fits so much goshdarned data!!!\n\nso anyway here's my top list of girls i think are cute:\n\n1. dx. cute!!!\n2. can't say, she'll be mad at me ;_;\n3. this girl will also be mad at me if i say.\n4. puck. very cute girl!!\n5. ely. claims to not be girl but is cute nevertheless\n6. ypad. cute girl who goes on planes\n7. suzy. cute quake girl!!\n\nchrist almighty am i STILL allowed to go on??? how big is this thing??\n\nokay i , i guess i'll keep going then.\n\n8. vivian. cute bug girl!!!\n9. hikari. cute girl eat a pepsi\n10. me?? can i be the cute girl??? i hope so... maybe one day. ohh, to be the cute girl..... my heart aches ;3;\n\nso uh. how's everybody doing? i'm pretty pleased with how i made the qr codes work, personally. but like that's just me. i realize that we all have our separate struggles in life... not all girls are able to render qr codes. but i am. and actually that might be sort of a USP for when i'm dating? not everyone can say their gf made a qr code by herself. so maybe i get to be like a trophy gf?? well, a girl can dream...\n\nmaybe i should make a substack or something. maybe i can get people to pay me to listen to my stream of consciousness.\n\nwell anyway! hope you're all doing good out there. take care and i'll uhhhhhh keep posting about qr codes in the meantime i guess i don't really have much else going on.\n\nmwah mwah, xoxo...\n- es"),
        // (ASCII,"me when she points a gun at me O_o;;"),

        // (2,"this is ascii! >w<"),
        // (2, "hi :)"),
        //
    ];

    // generate_codeword_table();

    // for mask in 0..7 {
    //     _debug_print_qr(&make_qr(mode_data, 7, 0, mask));
    //     println!("mask {}\n\n", mask);
    // }

    // _debug_print_qr(&make_qr(mode_data, None, 0, 3));
    println!("{}", (make_qr(mode_data, None, 0, None)).as_xbm("big"));

    // gen_qr_using_modes(Some(mode_data));
}

fn generate_inverse_codeword_table() {
    let mut vec = Vec::new();

    for version in 0..40 {
        for level in 0..=3 {
            vec.push((DATA_CODEWORDS[version][level], version, level));
        }
    }

    vec.sort_by_key(|x| x.0);

    print!("pub const INVERSE_DATA_CODEWORDS: [(u32,(u32,u32));160] = [");
    for i in &vec {
        let (x, version, level) = i;
        print!("({:04},({:02},{})),", x, version + 1, level);
    }
    println!("];");
}

fn generate_codeword_table() {
    print!("pub const DATA_CODEWORDS:[[u32;40];4]=[");
    for level in 0..=3 {
        print!("[");
        for version in 1..=40 {
            let (bc, _, dcw, opt) = get_block_info(version, level);
            let data_limit = if let Some((bc2, _, dcw2)) = opt {
                bc * dcw + bc2 * dcw2
            } else {
                bc * dcw
            };
            print!("{:04},", data_limit);
        }
        print!("],");
    }
    println!("];");
}

fn gen_qr_using_modes(custom_input: Option<&[(Mode, &str)]>) {
    let version = 1;
    let cwords = CODEWORDS[version as usize - 1];
    let ecwords = 7;
    for mask in 0..=7 {
        let message: &mut Badstream = &mut invoke_modes(
            {
                if let Some(goods) = custom_input {
                    goods
                } else {
                    &[
                        // (2, "this is ASCII mode! it has 255 chars. "),
                        // (
                        //     1,
                        //     "THIS IS ALPHANUMERIC. IT HAS 45 CHARS. AND NUMERIC MODE ONLY HAS 10: ",
                        // ),
                        // (0, "01234565789"),
                        // (1, "THIS IS ALPHANUMERIC MODE"),
                        // (1, "$$$$$$$$$$$$$$$$$$$$$$$$$"),
                        (ASCII, "this is ascii >w<"),
                        // (0, "12345678901234567890123456789012345678901"),
                        // (0, "111111111111111111111111111111111111111111"),
                        // (2, "the constant Pi is approximately 3."),
                        // (0, "1415926"),
                        // (1, " ... AND I AM SO ANGRY AB"),
                        // (2, "out it... no i'm Calm now :)"),
                    ]
                }
            },
            version,
        );

        println!("--- unpadded length is {} + terminator", message.len() - 4);
        // println!("contents:\n   {}", {
        //     let mut string = String::new();
        //     for i in (&message).iter() {
        //         string.push(if *i { '1' } else { '0' });
        //     }
        //     string
        // });

        // this took so long to fix...
        if message.len().div_ceil(8) > (cwords - ecwords) as usize {
            panic!(
                "{} too many message codewords:\n   allowed: {}\n   message: {}",
                (message.len().div_ceil(8)) - (cwords - ecwords) as usize,
                (cwords - ecwords) as usize,
                (message.len().div_ceil(8))
            )
        }

        /*  for p in message {
            print!("{}", u8::from(*p));
        } */
        println!();

        /* if false */
        {
            // let mask = todo!();
            let bitmap = &mut ImgRowAligned::new_blank_qr(version);

            pad_to((cwords - ecwords) as usize, message);
            let code = encode_message(&badstream_to_polynomial(message), ecwords as u32);

            let mut count = 0;
            for (a, &i) in (&code).iter().enumerate() {
                print!("{:02X} ", i);
                count += 1;
                if a + 1 == (cwords - ecwords) as usize {
                    println!("\n");
                    count = 0;
                } else if count % 8 == 7 {
                    println!();
                    count = 0;
                }
            }
            println!();

            let encoded_message = &polynomial_to_badstream(&code);
            // {
            //     let encoded_message_2: &mut Badstream = &mut Vec::new();
            //     push_codewords(
            //         code.iter()
            //             .map(|&x| x as u8)
            //             .collect::<Vec<u8>>()
            //             .as_slice(),
            //         encoded_message_2,
            //     );
            //     assert!(*encoded_message == *encoded_message_2);
            // }
            set_fcode(
                bitmap,
                version,
                (0, 0),
                data_to_fcode([0b01, 0b00, 0b11, 0b10][0], mask).unwrap(),
            );
            write_badstream_to_bitmap(encoded_message, bitmap);
            bitmap.qr_mask_xor(mask);
            _debug_print_qr(bitmap);
            println!("{}", bitmap.qr_penalty());
            println!();
        }
    }
}

#[test]
fn test_penalty() {
    // essentially test to make sure the penalty function is monotonic
    // for adding black pixels to a completely white square
    let mask = &mut ImgRowAligned::new(125, 125);

    let mut old = mask.qr_penalty();
    let mut new: u32;
    for i in 0..125 {
        mask.set_bit((77 * i) % 125, i, true);
        new = mask.qr_penalty();
        assert!(new < old);
        old = new;
    }
}

fn gen_rdsm_polynomials() {
    let mut a = Vec::new();
    for i in ERROR_CORRECTION_CODEWORDS {
        a.push(make_rdsm_generator_polynomial(i));
    }
    print!("&[");
    for poly in &a {
        print!("&[");
        for &el in poly {
            let k = log(el);
            print!("{},", k);
        }
        print!("],");
    }
    println!("];");
}

fn gen_codeword_table() {
    let mut cwords = Vec::new();
    for version in 1..=40 {
        let mut cwordcount: usize = 1;

        // let bitmap = ImgRowAligned::new_blank_qr(version);
        let mut x = (version_to_size(version).unwrap() - 1) as usize;
        let mut y = x;

        while let Some((x2, y2)) = next_data_bit(x, y, version) {
            (x, y) = (x2, y2);
            cwordcount += 1;
        }
        cwords.push(cwordcount / 8);
    }
    println!("{:?}", cwords);
}

fn first_qr_code() {
    for mask in 0..=7 {
        let message: &mut Badstream = &mut Vec::new();
        let text = "hi :)";
        let bitmap = &mut ImgRowAligned::new_blank_qr(5);

        push_bits("0100", message);
        push_bits(format!("{:08b}", text.len()).as_str(), message);
        push_ascii(text, message);
        push_bits("0000", message);
        pad_to(134 - 26, message);
        let code = encode_message(&badstream_to_polynomial(message), 26);

        for (a, &i) in (&code).iter().enumerate() {
            print!("{:02X} ", i);
            if a % 8 == 7 {
                println!();
            }
        }
        println!();

        let encoded_message: &mut Badstream = &mut Vec::new();
        push_codewords(
            code.iter()
                .map(|&x| x as u8)
                .collect::<Vec<u8>>()
                .as_slice(),
            encoded_message,
        );
        set_fcode(bitmap, 5, (0, 0), data_to_fcode(0b01, mask).unwrap());
        write_badstream_to_bitmap(encoded_message, bitmap);
        bitmap.qr_mask_xor(mask);
        _debug_print_qr(bitmap);
        println!();
    }
}

fn qr_correctness_check() {
    let mut hello = testutil::hello1();
    hello.unmask();
    _debug_print(&hello);
    let mut bits: Badstream = Vec::new();
    let (mut x, mut y) = (20, 20);
    bits.push(hello.get_bit(x, y).unwrap());
    while let Some((x2, y2)) = next_data_bit(x, y, hello.qr_version().unwrap()) {
        (x, y) = (x2, y2);
        bits.push(hello.get_bit(x, y).unwrap());
    }
    for (o, &i) in (&bits).iter().enumerate() {
        print!("{}", u8::from(i));
        if o % 8 == 7 {
            println!();
        } else if o % 2 == 1 {
            print!("-");
        }
    }
    println!();
    let p = badstream_to_polynomial(&bits);
    for &i in &p {
        print!("{i:02X} ");
    }
    println!();
    for &i in &p {
        print!("{i:08b} ");
    }
    println!();
    let err = 10;
    println!("error-correcting codewords: {}", err);
    let divisor = make_rdsm_generator_polynomial(err as u32);
    println!("divisor:");
    prettyprint(&divisor);
    let result = polynomial_remainder(&p, &divisor);
    println!("result:");
    _doubleprint(&result);
    println!("{:?}", &result);
}

fn read_bitstream() {
    let mut code = testutil::hello1();
    code.qr_mask_xor(
        interpret_format(get_fcode(&code, 1, (0, 0)).unwrap())
            .unwrap()
            .1,
    );
    let (mut x, mut y) = (20, 20);
    for _i in 0..280 {
        print!("{}", u8::from(code.get_bit(x, y).unwrap()));
        if let Some(coords) = next_data_bit(x, y, 1) {
            (x, y) = coords;
        } else {
            break;
        }
    }
}

fn _compare_mask_to_isdata() {
    let mask = testutil::mask();
    let mut blank = testutil::blank();
    let size = version_to_size(1).unwrap() as usize;
    for x in 0..size {
        for y in 0..size {
            if mask.get_bit(x, y).unwrap() != coord_is_data(x, y, 1) {
                // println!("error: x {}, y {}", x, y);
                blank.set_bit(x, y, true);
            }
        }
    }
    _debug_print(&mask);
    println!();
    _debug_print(&blank);
}

fn _full_squiggle_test() {
    let mut coordpairs = [[0; 256]; 256];
    let mut a = Img::new(21, 21);
    let (mut cx, mut cy) = (20, 20);
    a.invert();
    for _i in 0..(8 * 28) {
        a.set_bit(cx, cy, false);
        if let Some(new_coords) = next_data_bit(cx, cy, 1) {
            print!("{:?} -> {:?}", (cx, cy), new_coords);
            (cx, cy) = new_coords;

            let a = &mut coordpairs[cx][cy];
            if *a != 0 {
                println!(" {}!!", a);
            } else {
                println!();
            }
            *a += 1;
        } else {
            break;
        }
    }
    _debug_print(&a);
}

fn _bugtest_squiggle(version: u32) {
    let size = version_to_size(version).unwrap() as usize;
    let mut coordpairs: Vec<Vec<&str>> = Vec::new();
    coordpairs.resize(size, {
        let mut gunk = Vec::new();
        gunk.resize(size, "⬜️");
        gunk
    });
    // let mask = testutil::mask();
    // debug_print(&mask);
    for x1 in 0..size {
        for y1 in 0..size {
            if coord_is_data(x1, y1, version) {
                coordpairs[x1][y1] = {
                    if let Some((x2, y2)) = next_data_bit(x1, y1, version) {
                        let xdiff = x2 as i32 - x1 as i32;
                        let ydiff = y2 as i32 - y1 as i32;

                        if xdiff.abs() > 1 || ydiff.abs() > 1 {
                            "🦅"
                        } else if xdiff.abs() == ydiff.abs() {
                            // diagonal, or none
                            match (xdiff.signum(), ydiff.signum()) {
                                (1, 1) => "↘️ ",
                                (1, -1) => "↗️ ",
                                (-1, 1) => "↙️ ",
                                (-1, -1) => "↖️ ",
                                _ => "💥",
                            }
                        } else if xdiff.abs() > ydiff.abs() {
                            // horizontal-ish
                            if xdiff.signum() == 1 {
                                "➡️ "
                            } else {
                                "⬅️ "
                            }
                        } else {
                            // vertical-ish
                            if ydiff.signum() == -1 {
                                "⬆️ "
                            } else {
                                "⬇️ "
                            }
                        }
                    } else {
                        "💥"
                    }
                };
            }
        }
    }
    for y in 0..size {
        let mut a = "".to_string();
        for x in 0..size {
            a.push_str(coordpairs[x][y]);
        }
        println!("{}", a);
    }
}

fn _highlight_codewords(version: u32) {
    let size = version_to_size(version).unwrap() as usize;
    let mut coordpairs: Vec<Vec<&str>> = Vec::new();
    coordpairs.resize(size, {
        let mut gunk = Vec::new();
        gunk.resize(size, "⬜️");
        gunk
    });
    // let mask = testutil::mask();
    // debug_print(&mask);
    let (mut x, mut y) = (size - 1, size - 1);
    let mut colors = ["🟥", "🟧", "🟨", "🟩", "🟦", "🟪"]
        .iter()
        .cycle()
        .peekable();

    for (count, _i) in (0..size.pow(2)).enumerate() {
        if count % 8 == 0 {
            colors.next();
        }

        coordpairs[x][y] = colors.peek().unwrap();

        if let Some((x2, y2)) = next_data_bit(x, y, version) {
            (x, y) = (x2, y2);
        } else {
            break;
        }
    }

    for y in 0..size {
        let mut a = "".to_string();
        for x in 0..size {
            a.push_str(coordpairs[x][y]);
        }
        println!("{}", a);
    }
}

fn _print_symbol_diagram(version: u32) {
    let size = version_to_size(version).unwrap() as usize;
    let mut coordpairs: Vec<Vec<&str>> = Vec::new();
    coordpairs.resize(size, {
        let mut gunk = Vec::new();
        gunk.resize(size, "");
        gunk
    });
    for x in 0..size {
        for y in 0..size {
            coordpairs[x][y] = match coord_status(x, y, version) {
                0 => "🟩",
                1 => "🟨",
                2 => "🟥",
                3 => "🟦",
                4 => "⬜️",
                _ => "⬛️",
            };
        }
    }
    for y in 0..size {
        let mut a = "".to_string();
        for x in 0..size {
            a.push_str(coordpairs[x][y]);
        }
        println!("{}", a);
    }
}

fn _test_polynomial_mult() {
    // it works!!
    let cafebabe: Polynomial = Vec::from([0xca, 0xfe, 0xba, 0xbe]);
    let deadbeef: Polynomial = Vec::from([0xde, 0xad, 0xbe, 0xef]);
    prettyprint(&cafebabe);
    prettyprint(&deadbeef);
    println!("{:?}", polynomial_multiply(&cafebabe, &deadbeef));
    println!("{:?}", polynomial_multiply(&deadbeef, &cafebabe));
    println!("{:?}", es_polynomial_multiply(&cafebabe, &deadbeef));
    println!("{:?}", es_polynomial_multiply(&deadbeef, &cafebabe));
}

fn _doubleprint(input: &Polynomial) {
    charprint(input);
    prettyprint(input);
    println!();
}

fn _test_polynomial_div() {
    let cafebabe: Polynomial = Vec::from([0xca, 0xfe, 0xba, 0xbe]);
    // let deadbeef: Polynomial = Vec::from([0xde, 0xad, 0xbe, 0xef]);
    let big_1: Polynomial = (1..10).map(|x| (x * 541) % 256).collect();
    let big_2: Polynomial = (1..15).map(|x| (x * 311) % 256).collect();
    let sum = polynomial_add(&big_1, &big_2);
    let rem_1 = polynomial_remainder(&big_1, &cafebabe);
    let rem_2 = polynomial_remainder(&big_2, &cafebabe);
    let rem_sum = polynomial_add(&rem_1, &rem_2);
    let sum_rem = polynomial_remainder(&sum, &cafebabe);

    println!("divisor:");
    _doubleprint(&cafebabe);
    // prettyprint(&deadbeef);

    println!("\nbig polynomial 1:");
    _doubleprint(&big_1);
    println!("big polynomial 2:");
    _doubleprint(&big_2);
    println!("sum of bigs:");
    _doubleprint(&sum);

    println!("\nremainder of big 1:");
    _doubleprint(&rem_1);
    println!("remainder of big 2:");
    _doubleprint(&rem_2);

    println!("\nremainder of sums:");
    _doubleprint(&sum_rem);
    println!("sum of remainders:");
    _doubleprint(&rem_sum);

    assert_eq!(sum_rem, rem_sum);
}

fn _test_rdsm_generator() {
    for i in [7, 10, 13, 15, 20, 22, 24, 68] {
        let a = make_rdsm_generator_polynomial(i);
        print!("{} -- ", i);
        prettyprint(&a);
        println!();
    }
}

fn _remasking_test() {
    let code = xbm_filepath_into_bitmap("hellocode_smol.xbm");

    _debug_print_qr(&code);
    let penalties = code.qr_penalty_split();
    println!("penalty: {}", &penalties.iter().sum::<u32>());
    println!(
        "    adjacent: {}\n    block: {}\n    fake marker: {}\n    proportion: {}",
        penalties[0], penalties[1], penalties[2], penalties[3]
    );
    println!("\n");
    // let i = 0;
    /*
    for i in 0..=7 {
            if let Some(code2) = _qr_remask_v1_symbol(&code, i) {
                println!("mask {}", i);
                _debug_print_qr(&code2);
                let penalties = code2.qr_penalty_split();
                println!("penalty: {}", &penalties.iter().sum::<u32>());
                println!(
                    "    adjacent: {}\n    block: {}\n    fake marker: {}\n    proportion: {}",
                    penalties[0], penalties[1], penalties[2], penalties[3]
                );
                println!("\n");
            }
        }
     */
}
fn unmask(input: &mut ImgRowAligned) {
    let fcode = get_fcode(input, 1, (0, 0)).unwrap();
    let mask = interpret_format(fcode).unwrap().1;
    input.qr_mask_xor(mask);
}
// this function works perfectly!! it's great
fn _qr_remask_v1_symbol(input: &ImgRowAligned, mask_pattern: u8) -> Option<ImgRowAligned> {
    let old_fcode = get_fcode(input, 1, (0, 0))?;
    let (correction_level, old_mask_pattern) = interpret_format(old_fcode)?;
    if mask_pattern == old_mask_pattern {
        return Some(input.clone());
    }

    let pixelmask = xbm_filepath_into_bitmap("hellomask_smol.xbm");

    let mut image = input.clone();
    image.qr_mask_xor(old_mask_pattern);
    image.qr_mask_xor(mask_pattern);
    image.mask_set(input, &pixelmask);
    let fcode = data_to_fcode(correction_level, mask_pattern).unwrap();
    // println!("fcode\n{:015b}\n\nold fcode\n{:015b}\n", fcode, old_fcode);
    set_fcode(&mut image, 1, (0, 0), fcode);

    Some(image)
}

fn _print_qr_mask_patterns() {
    let x = ImgRowAligned::new(25, 25);
    // let x = image_type::continuous::Img::new(25,25);
    for i in 0..8 {
        let mut masky = x.clone();
        masky.qr_mask_xor(i);
        println!();
        _debug_print(&masky);
    }
}

// fn check_format_pattern() {
//     let mut test_img = image_type::rowaligned::ImgRowAligned::new(25, 25);
//     let qr = xbm_path_convert("hellocode.xbm");

//     for i in 0..=14 {
//         let ((a, b), (c, d)) = image_type::qr_standard::format_info_coords(1, i).unwrap();
//         test_img.set_bit(a + 2, b + 2, true);
//         test_img.set_bit(c + 2, d + 2, true);
//     }

//     debug_print(&test_img);
//     println!();
//     debug_print(&qr);
// }

fn _test_format_parsing(path: &str) {
    let xbm_string = std::fs::read_to_string(path).unwrap();
    let xbm_bitmap = ImgRowAligned::from_xbm(&xbm_string).unwrap();
    let fcode = get_fcode(&xbm_bitmap, 1, (0, 0)).unwrap();
    println!("{:#b}", fcode);
    println!("remainder {:#b}", qr_fcode_remainder(fcode as u32));

    {
        let (correction, mask) = interpret_format(fcode).unwrap();

        println!("error correction {}", {
            match correction {
                1 => 'L',
                2 => 'M',
                3 => 'Q',
                4 | _ => 'H',
            }
        });
        println!("masking pattern {:#05b}", mask);
        _debug_print(&xbm_bitmap);
        /* for mask in  0..=7 */
        {
            let mut code_for_masking = xbm_bitmap.clone();
            let pixelmask = {
                let mut x = xbm_filepath_into_bitmap("hellomask_smol.xbm");
                x.invert();
                x
            };
            let xor_mask_pattern = {
                let (width, height) = code_for_masking.dims();
                let mut x = ImgRowAligned::new(width, height);
                x.qr_mask_xor(mask);
                x
            };

            // debug_print(&code_for_masking);
            println!();
            code_for_masking.qr_mask_xor(mask);
            // for i in 0..goop.dims().1 {
            //     println!("{}", debug_print_row(&goop, i, true).unwrap());
            // }
            // println!();
            code_for_masking.mask_set(&xbm_bitmap, &pixelmask);
            println!("mask {}:", mask);
            _debug_print(&xor_mask_pattern);
            println!();
            _debug_print(&code_for_masking);
        }
    }
}

fn _debug_print<T: Bitmap>(input: &T) {
    for y in 0..input.dims().1 {
        println!("{}", debug_print_row(input, y, true).unwrap())
    }
}

fn _debug_print_qr<T: Bitmap>(input: &T) {
    let throwaway_hack = || {
        for _i in 0..2 {
            for _y in 0..input.dims().1 + 4 {
                print!("⬜️");
            }
            println!()
        }
    };

    throwaway_hack();
    for y in 0..input.dims().1 {
        println!("⬜️⬜️{}⬜️⬜️", debug_print_row(input, y, true).unwrap())
    }
    throwaway_hack();
}

fn debug_print_row<T: Bitmap>(input: &T, y: usize, emoji: bool) -> Option<String> {
    let row = input.get_row(y)?;
    let mut output = String::new();
    for j in (0..input.dims().0).rev() {
        if emoji {
            output.push_str(if ((row >> j) % 2) == 1 {
                "⬛️"
            } else {
                "⬜️"
            })
        } else {
            output.push(if ((row >> j) % 2) == 1 { '1' } else { '0' })
        };
    }
    Some(output)
}

fn _test_xbm_output() {
    println!(
        "{}",
        ImgRowAligned::as_xbm(
            &{
                let x =
                    ImgRowAligned::from_xbm(std::fs::read_to_string("es.xbm").unwrap().as_str())
                        .unwrap();
                // x.invert();
                x
            },
            "cool",
        )
    );
}

fn _test_xbm(path: &str) {
    let input = std::fs::read_to_string(path).unwrap();
    let x = ImgRowAligned::from_xbm(&input).unwrap();
    let mut vector: Vec<ImgRowAligned> = Vec::new();
    vector.push(x.clone());
    for i in 0..=7 {
        let mut masked = x.clone();
        masked.qr_mask_xor(i);
        vector.push(masked);
    }
    for x in vector {
        for i in 0..x.dims().1 {
            println!("{}", debug_print_row(&x, i, true).unwrap());
        }
        println!();
    }
}

fn xbm_filepath_into_bitmap(path: &str) -> ImgRowAligned {
    let input = std::fs::read_to_string(path).unwrap();
    ImgRowAligned::from_xbm(&input).unwrap()
}
// tests qr format check, assuming debug printing is enabled
fn _test_checkfmt() {
    for i in 10..20 {
        qr_fcode_remainder((2u32.pow(15) - 20) + i);
        println!();
    }
}

// just the example taken from the tutorial
// returns 0001010001111010 and 0000000011000011 (correct)
#[test]
pub fn _test_gf() {
    /*
        >>> a = 0b10001001
        >>> b = 0b00101010
        >>> print bin(gf_mult_noLUT(a, b, 0)) # multiplication only
        0b1010001111010
        >>> print bin(gf_mult_noLUT(a, b, 0x11d)) # multiplication + modular reduction
        0b11000011
    */
    let a = 0b10001001;
    let b = 0b00101010;
    // println!("{:016b}", galois_multiply(a, b, 0));
    // println!("{:016b}", galois_multiply(a, b, QR_CODEWORD_GEN));

    // works!
    assert!(galois_multiply(a, b, 0) == 0b0001010001111010);
    assert!(galois_multiply(a, b, QR_CODEWORD_GEN) == 0b0000000011000011);
    assert!(table_multiply(a, b) == 0b0000000011000011);

    println!("basic tests passed! now here's the real trial:");

    let mut hits = [false; 255];

    for i in 0..255 {
        let k = table_pow(0b10, i);
        assert!(log(k) == i as usize);
        hits[(k - 1) as usize] = true;
    }
    assert!(!hits.contains(&false));

    println!("you are a master multiplication table !!");

    for x in 0..255 {
        for y in 0..x {
            let a =
                galois_multiply(x, y, QR_CODEWORD_GEN) == galois_multiply(y, x, QR_CODEWORD_GEN);
            let b = galois_multiply(x, y, QR_CODEWORD_GEN) == table_multiply(x, y);
            let c = {
                if x * y != 0 {
                    (log(x) + log(y)) % 255 == log(galois_multiply(x, y, QR_CODEWORD_GEN))
                } else {
                    true
                }
            };
            if !(a && b && c) {
                println!("({:03},{:03}) failed {}", x, y, {
                    let mut text = String::new();
                    if !a {
                        text.push('a')
                    }
                    if !b {
                        text.push('b')
                    }
                    if !c {
                        text.push('c')
                    }
                    text
                });
            }
        }
    }

    println!("wowza!!");

    // println!("{:016b}", galois_multiply_peasant_full(a, b, 0, 256, true));
    // println!(
    //     "{:016b}",
    //     galois_multiply_peasant_full(a, b, 0x11d, 256, true)
    // );
}

fn _test_reed_solomon(test: u8) {
    // time to generate a qr code (clueless)
    let mut lookup_tables = BLANK_EXP_LOG_LUTS;
    generate_exp_log_tables(&mut lookup_tables);

    if test & 0b1 != 0 {
        println!("\n\n{:?}\n{:?}\n\n", lookup_tables.0, lookup_tables.1);
    }

    if test & 0b10 != 0 {
        let input: Polynomial = Vec::from(TEST_MSG);
        let control: Polynomial = Vec::from(FULL_TEST_RESULT);
        let output = encode_message(&input, 10);
        assert_eq!(output, control);

        println!("input:");
        charprint(&input);
        prettyprint(&input);
        println!("output:");
        charprint(&output);
        prettyprint(&output);
        println!("control:");
        charprint(&control);
        prettyprint(&control);
    }

    if test & 0b100 != 0 {
        for i in 1..255 {
            print!("{:3}:", i);
            for j in 1..255 {
                // let mul1 = galois_multiply(i as Element, j as Element, QR_CODEWORD_GEN);
                // let mul2 = table_multiply(i as Element, j as Element);

                // let div2 = table_divide(i as Element, j as Element, &lookup_tables);

                // let mut status: u8 = 0;

                if i as Element
                    != galois_multiply(
                        table_divide(i as Element, j as Element),
                        j as Element,
                        QR_CODEWORD_GEN,
                    )
                {
                    print!("x");
                }
                // if div1 != div2 {
                //     status += 2;
                // }

                // print!("{}", status);
                //     if mul1 != mul2 {
                //         all_good = false;
                //         println!(
                //             "\n{:#08b} * {:#08b} =\n{:#08b}\n{:#08b}\n",
                //             i, j, mul1, mul2
                //         );
                //     }
                //     if div1 != div2 {
                //         all_good = false;
                //         println!(
                //             "\n{:#08b} / {:#08b} = \n{:#08b}\n{:#08b}\n",
                //             i, j, div1, div2
                //         );
                //     }
            }
            println!();
        }
    }

    if test & 0b1000 != 0 {
        // prints a table of the powers of 2 mod PRIM, in decimal
        let nice = false;

        if nice {
            print!("-----");
            for n in 0..(256 / 16) {
                print!("--{:2}", n);
            }
        }

        for i in 0..256 {
            if i % 16 == 0 {
                println!();
                if nice {
                    print!("{:3} - ", i);
                }
            }
            print!("{:3} ", lookup_tables.0[i]);
        }
        println!();
    }

    if test & 0b10000 != 0 {
        println!("{:?}", &(lookup_tables.0)[..256]);
    }

    if test & 0b100000 != 0 {}
}

mod testutil {
    use super::*;

    pub fn mask() -> ImgRowAligned {
        xbm_filepath_into_bitmap("hellomask_smol.xbm")
    }
    pub fn hello1() -> ImgRowAligned {
        xbm_filepath_into_bitmap("hellocode_smol.xbm")
    }
    pub fn hello2() -> ImgRowAligned {
        xbm_filepath_into_bitmap("hellocode2_smol.xbm")
    }
    pub fn blank() -> ImgRowAligned {
        ImgRowAligned::new(21, 21)
    }
}
