use rdsm::{galois_multiply, table_divide, table_multiply, Element, PRIM};

mod bitmask;
mod export;
mod image_type;
mod rdsm;
mod zigzag;

// program flow:
// generate blank matrix
// write a message to it
//   (what format is the message??)
// along with error correction bits
//   (are those included in the message,
//    or separate?)
// write format information to it
//   (at what step, what format?)
// apply bitmask
//   (which mask????)
// output

// re: format of the message, i want it to be
// a list of bytes, like 0f a6 42 etc.

fn main() {
    test_reed_solomon(0b1000);
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

// tests qr format check, assuming debug printing is enabled
fn test_checkfmt() {
    for i in 10..20 {
        rdsm::qr_check_fcode((2u32.pow(15) - 20) + i);
        println!();
    }
}

fn test_reed_solomon(test: u8) {
    // time to generate a qr code (clueless)
    let mut lookup_tables = rdsm::BLANK_EXP_LOG_LUTS;
    rdsm::generate_exp_log_tables(&mut lookup_tables, rdsm::PRIM);

    if test & 0b1 != 0 {
        println!("\n\n{:?}\n{:?}\n\n", lookup_tables.0,lookup_tables.1);
    }

    if test & 0b10 != 0 {
        let input: rdsm::Polynomial = Vec::from(rdsm::TEST_MSG);
        let mut control: rdsm::Polynomial = Vec::from(rdsm::FULL_TEST_RESULT);
        let mut output = rdsm::encode_message(&input, 10, &lookup_tables);
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
