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
    // time to generate a qr code (clueless)

    let a: rdsm::polynomial = Vec::from(rdsm::TEST_MSG);

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
