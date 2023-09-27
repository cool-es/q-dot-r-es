use rdsm::*;
use image_type::Bitmap;

mod bitmask;
mod export;
mod image_type;
mod rdsm;
mod zigzag;

// what we know:
// 1. table-based operations are correct, seemingly regarless of if you use mod 255 or mod 256
// 2. qr_gen has a mysteriously short cycle, 2^15 = 1
// 3. prim has a cycle of length 255

// branch to implement various qr standard-specific
// routines without getting in the way of debugging
// the galois field operations

fn main() {
    test_xbm("hellocode.xbm");

    // test_xbm_output();

    //wikiv::test_gf();

    /* for k in 0..5 {
        let i = 1 << k;
        println!("{:05b} -> {:015b} ({})", i, wikiv::encode_format(i), i);
    } */

    //test_checkfmt();
    /* let mut codes: Vec<u32> = Vec::new();
    codes.resize(32, 0);

    println!("test encode:");
    for i in 0..32 {
        codes[i as usize] = wikiv::qr_generate_fcode(i);
        println!("{:05b} -> {:015b} ({})", i, codes[i as usize], i);
    }
    println!();
    for i in 0..32 {
        let format = wikiv::qr_generate_fcode(i);
        println!(
            "{:2}: format {:015b}, rem. {}",
            i,
            format,
            wikiv::qr_check_fcode(format)
        ); */
    /* print!("{:2}: ", i);
        for j in 0..i {
            print!("{} ", (codes[i] ^ codes[j]) ^ codes[i ^ j]);
            /* if (codes[i] ^ codes[j]).count_ones() > 8 {
                println!(
                    "{:2} vs {:2}: distance {}",
                    i,
                    j,
                    (codes[i] ^ codes[j]).count_ones()
                );
            } */
        }
    } */
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
        image_type::rowaligned::ImgRowAligned::as_xbm(
            &{
                let mut x = image_type::rowaligned::ImgRowAligned::from_xbm(
                    std::fs::read_to_string("es.xbm").unwrap().as_str(),
                )
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
    let x = image_type::rowaligned::ImgRowAligned::from_xbm(&input).unwrap();
    let mut vector: Vec<image_type::rowaligned::ImgRowAligned> = Vec::new();
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

// tests qr format check, assuming debug printing is enabled
fn test_checkfmt() {
    for i in 10..20 {
        qr_check_fcode((2u32.pow(15) - 20) + i);
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
    println!("{:016b}", galois_multiply(a, b, 0));
    println!("{:016b}", galois_multiply(a, b, 0x11d));

    println!("{:016b}", galois_multiply_peasant_full(a, b, 0, 256, true));
    println!(
        "{:016b}",
        galois_multiply_peasant_full(a, b, 0x11d, 256, true)
    );
}

fn test_reed_solomon(test: u8) {
    // time to generate a qr code (clueless)
    let mut lookup_tables = BLANK_EXP_LOG_LUTS;
    generate_exp_log_tables(&mut lookup_tables, PRIM);

    if test & 0b1 != 0 {
        println!("\n\n{:?}\n{:?}\n\n", lookup_tables.0, lookup_tables.1);
    }

    if test & 0b10 != 0 {
        let input: Polynomial = Vec::from(TEST_MSG);
        let mut control: Polynomial = Vec::from(FULL_TEST_RESULT);
        let mut output = encode_message(&input, 10, &lookup_tables);
        //assert!(output == control);
        println!("output:\n{:?}\ncontrol:\n{:?}", &output, &control);

        let len = std::cmp::max(output.len(), control.len());
        control.resize(len, 0);
        output.resize(len, 0);
        println!("difference:");
        for i in 0..len {
            if i == 16 {
                println!();
            }
            print!("{}", output[i] as i32 - control[i] as i32);
            if i != len - 1 {
                print!(", ");
            }
        }
        println!();
    }

    if test & 0b100 != 0 {
        for i in 1..255 {
            print!("{:3}:", i);
            for j in 1..255 {
                let mul1 = galois_multiply(i as Element, j as Element, PRIM);
                let mul2 = table_multiply(i as Element, j as Element, &lookup_tables);

                // let div2 = table_divide(i as Element, j as Element, &lookup_tables);

                // let mut status: u8 = 0;

                if i as Element
                    != galois_multiply(
                        table_divide(i as Element, j as Element, &lookup_tables),
                        j as Element,
                        PRIM,
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
