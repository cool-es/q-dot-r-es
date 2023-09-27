use image_type::Bitmap;

mod bitmask;
mod export;
mod image_type;
mod rdsm;
mod zigzag;

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
        rdsm::qr_check_fcode((2u32.pow(15) - 20) + i);
        println!();
    }
}

fn test_reed_solomon() {
    // time to generate a qr code (clueless)
    let mut lookup_tables = rdsm::BLANK_EXP_LOG_LUTS;
    rdsm::generate_exp_log_tables(&mut lookup_tables, rdsm::PRIM);
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
