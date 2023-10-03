mod image;
mod rdsm;

use image::*;
use rdsm::*;

fn main() {
    {
        let a = string_to_polynomial("Ough,, Houhgh");
        // let b = string_to_polynomial("foghorn leghorn");
        doubleprint(&a);
        // doubleprint(&b);

        let b = encode_message(&a, 20);
        doubleprint(&b);

        let c = polynomial_remainder(&b, &make_rdsm_generator_polynomial(20));
        doubleprint(&c);
        // let ab = polynomial_multiply(&a, &b);
        // doubleprint(&ab);

        // let div = polynomial_remainder(&a, &b);
        // doubleprint(&div);
    }
    // test_polynomial_mult();
    // test_polynomial_div();
    // _print_exp_log_tables();
    // test_gf();
    // test_reed_solomon(0b10);
    // remasking_test();
    // test_rdsm_generator();
}

fn test_polynomial_mult() {
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

fn doubleprint(input: &Polynomial) {
    charprint(input);
    prettyprint(input);
    println!();
}

fn test_polynomial_div() {
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
    doubleprint(&cafebabe);
    // prettyprint(&deadbeef);

    println!("\nbig polynomial 1:");
    doubleprint(&big_1);
    println!("big polynomial 2:");
    doubleprint(&big_2);
    println!("sum of bigs:");
    doubleprint(&sum);

    println!("\nremainder of big 1:");
    doubleprint(&rem_1);
    println!("remainder of big 2:");
    doubleprint(&rem_2);

    println!("\nremainder of sums:");
    doubleprint(&sum_rem);
    println!("sum of remainders:");
    doubleprint(&rem_sum);

    assert_eq!(sum_rem, rem_sum);
}

fn test_rdsm_generator() {
    for i in [7, 10, 13, 15, 20, 22, 24, 68] {
        let a = make_rdsm_generator_polynomial(i);
        print!("{} -- ", i);
        prettyprint(&a);
        println!();
    }
}

fn remasking_test() {
    let code = xbm_filepath_into_bitmap("hellocode_smol.xbm");

    debug_print_qr(&code);
    println!();
    let i = 0;
    for i in 0..=7 {
        if let Some(code2) = qr_remask_v1_symbol(&code, i) {
            println!("mask {}", i);
            debug_print_qr(&code2);
        }
        println!("\n");
    }
}

// this function works perfectly!! it's great
fn qr_remask_v1_symbol(input: &ImgRowAligned, mask_pattern: u8) -> Option<ImgRowAligned> {
    let old_fcode = get_fcode(input, 1, (0, 0))?;
    let (correction_level, old_mask_pattern) = interpret_format(old_fcode)?;
    if mask_pattern == old_mask_pattern {
        return None;
    };

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

fn print_qr_mask_patterns() {
    let x = ImgRowAligned::new(25, 25);
    // let x = image_type::continuous::Img::new(25,25);
    for i in 0..8 {
        let mut masky = x.clone();
        masky.qr_mask_xor(i);
        println!();
        debug_print(&masky);
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

fn test_format_parsing(path: &str) {
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
        debug_print(&xbm_bitmap);
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
            debug_print(&xor_mask_pattern);
            println!();
            debug_print(&code_for_masking);
        }
    }
}

fn debug_print<T: Bitmap>(input: &T) {
    for y in 0..input.dims().1 {
        println!("{}", debug_print_row(input, y, true).unwrap())
    }
}

fn debug_print_qr<T: Bitmap>(input: &T) {
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

fn test_xbm_output() {
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

fn test_xbm(path: &str) {
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
fn test_checkfmt() {
    for i in 10..20 {
        qr_fcode_remainder((2u32.pow(15) - 20) + i);
        println!();
    }
}

// just the example taken from the tutorial
// returns 0001010001111010 and 0000000011000011 (correct)
pub fn test_gf() {
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

fn test_reed_solomon(test: u8) {
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
        // println!("output:\n{:?}\ncontrol:\n{:?}", &output, &control);

        // let len = std::cmp::max(output.len(), control.len());
        // control.resize(len, 0);
        // output.resize(len, 0);
        // println!("difference:");
        // for i in 0..len {
        //     if i == 16 {
        //         println!();
        //     }
        //     print!("{}", output[i] as i32 - control[i] as i32);
        //     if i != len - 1 {
        //         print!(", ");
        //     }
        // }
        // println!();
    }

    if test & 0b100 != 0 {
        for i in 1..255 {
            print!("{:3}:", i);
            for j in 1..255 {
                let mul1 = galois_multiply(i as Element, j as Element, QR_CODEWORD_GEN);
                let mul2 = table_multiply(i as Element, j as Element);

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
