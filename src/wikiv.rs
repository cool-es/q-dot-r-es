use core::panic;

// functions from the wikiversity "reed-solomon codes for coders" article
const DEBUG: bool = false;

// the qr-specific generator polynomial, 0b10100110111
pub const QR_GEN: u32 = 0x537;
// recurring polynomial in the wikiversity article, unsure of its significance
pub const PRIM: u32 = 0x11d;

// returns the remainder of fmt divided by g in GF(2^8), assuming fmt < 2^15
// named "check format" because it returns qr format data:
/*
    The format code should produce a remainder of zero
    when it is divided by the generator of the code.
    This function can also be used to encode the 5-bit format information.
    encoded_format = (format<<10) + qr_check_format(format<<10)

    The process for checking the encoded information is similar to long
    division, but uses exclusive-or instead of subtraction. The format code
    should produce a remainder of zero when it is "divided" by the so-called
    generator of the code. QR format codes use the generator 10100110111.
    This process is demonstrated for the format information
    in the example code (000111101011001) below.
*/
pub fn qr_check_fcode(fcode: u32) -> u32 {
    let qr_gen = 0x537; // 0b10100110111
    let mut output = fcode;

    if DEBUG {
        println!("!! check format debug:   {:#032b} ({})", output, output);
    }
    for i in (0..=4).rev() {
        if DEBUG {
            println!("!!! loop, i = {}", i);
            println!("!! format =              {:#032b}", output);
            println!("!! (1 << (i + 10))) =    {:#032b}", (1 << (i + 10)));
            println!(
                "!! bitwise and:          {:#032b}",
                (output & (1 << (i + 10)))
            );
        }
        if (1 << (i + 10)) & output != 0 {
            // the 2^(i+10) bit of fmt is 1

            if DEBUG {
                println!("-- \"0 !=\" condition met!");
                println!("-- format =              {:#032b}", output);
                println!("-- g << i =              {:#032b}", qr_gen << i);
                println!("-- format = bitwise xor: {:#032b}", output ^ (qr_gen << i));
            }

            // add (without carry) g shifted by i
            // because g has a 1 in the highest, 2^10, bit,
            // this will always erase that bit of fmt.
            // in essence we're doing like, "lights out"
            // on the 2^14 to 2^10 bits, from high to low
            output ^= qr_gen << i;
        }
    }
    if DEBUG {
        println!("!! finished:");
        println!("!! check format output:  {:#032b} ({})", output, output);
    }
    output
}

// generates a 15-bit code from a 5-bit number
// observations:
//     format(i) ^ format(j) == format(i ^ j)
//     format(a) >> 10 == a
// these codes form its basis under xor:
//     00001 -> 000010100110111 (1)
//     00010 -> 000101001101110 (2)
//     00100 -> 001000111101011 (4)
//     01000 -> 010001111010110 (8)
//     10000 -> 100001010011011 (16)
// not sure of the significance of this...

// fmt * 2^10 + remainder of (fmt * 2^10) / g
pub fn qr_generate_fcode(fmt: u32) -> u32 {
    if fmt >= 32 {
        panic!();
    }
    (fmt << 10) | qr_check_fcode(fmt << 10)
}

// earlier "qr_decode_format"
pub fn qr_find_fmt(fcode: u32) -> Option<u32> {
    // looks complex, is actually very simple:
    // try every format, generate its format code,
    // check the format code against the input
    // (lowest difference wins). returns None
    // if there's a tie
    let mut best_fmt: Option<u32> = None;
    let mut best_dist = 15;
    for try_fmt in 0..32 {
        let try_fcode = qr_generate_fcode(try_fmt);
        let try_dist = (fcode ^ try_fcode).count_ones();
        if try_dist < best_dist {
            best_dist = try_dist;
            best_fmt = Some(try_fmt);
        } else if try_dist == best_dist {
            best_fmt = None;
        }
    }
    best_fmt
}

// carry-less multiplication in GF(2^8)
// "cl_mul"
pub fn carryless_multiply(x: u32, y: u32) -> u32 {
    let mut output = 0;
    // the (32 - y.leading_zeros()) is a silly
    // optimization, it can just as well be 32
    for bit in 0..=(32 - y.leading_zeros()) {
        // for every 1 bit in y, xor in a copy of x
        // note that "== 1" won't work here -
        // it can be any power of 2
        if (y & (1 << bit)) != 0 {
            output ^= x << bit;
        }
    }
    output
}

pub fn galois_multiply(x: u32, y: u32, prime: u32) -> u32 {
    if prime > 0 {
        carryless_divide(carryless_multiply(x, y), prime)
    } else {
        carryless_multiply(x, y)
    }
}

pub fn qr_multiply(x: u32, y: u32) -> u32 {
    galois_multiply(x, y, QR_GEN)
}

pub fn bit_length(n: u32) -> u32 {
    match n {
        0 => 0,
        _ => n.ilog2() + 1,
    }
}

pub fn carryless_divide(dividend: u32, divisor: u32) -> u32 {
    if bit_length(dividend) < bit_length(divisor) {
        return dividend;
    }

    let dnd_length = bit_length(dividend);
    let dsr_length = bit_length(divisor);
    let mut dnd = dividend;
    for i in (0..=(dnd_length - dsr_length)).rev() {
        if dnd & (1 << i + dsr_length - 1) != 0 {
            dnd ^= divisor << i;
        }
    }
    dnd
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

// uses russian peasant multiplication
// default values prim = 0 field_charac_full = 256, carryless = true
/*
    Galois Field integer multiplication using Russian Peasant Multiplication algorithm
    (faster than the standard multiplication + modular reduction).
    If prim is 0 and carryless=False, then the function produces the result for
    a standard integer multiplication (no carry-less arithmetics nor modular reduction).
*/
pub fn galois_multiply_peasant_full(
    x: u32,
    y: u32,
    prime: u32,
    field_charac_full: u32,
    carryless: bool,
) -> u32 {
    let mut x = x;
    let mut y = y;
    let mut output = 0;

    while y > 0 {
        if (y & 1) != 0 {
            output = if carryless { output ^ x } else { output + x };
        }
        y >>= 1;
        x <<= 1;
        if prime > 0 && x & field_charac_full != 0 {
            x = x ^ prime;
        }
    }
    output
}

// attempting to make a nicer peasant multiply...
// not sure what the field character is supposed to be, but i'm guessing 256
pub fn galois_multiply_peasant_qr(x: u32, y: u32) -> u32 {
    galois_multiply_peasant_full(x, y, QR_GEN, 256, true)
}

// NEW ADDITIONS BELOW: section "multiplication with logarithms" starts here

// "init_tables", freely interpreted
// i don't understand why this function would reach all entries in log...
pub fn generate_exp_log_tables(tables: &mut ([u32; 256], [u32; 256]), prime: u32) {
    let (exp, log) = tables;
    let mut x: usize = 1;
    for i in 0..256 {
        exp[i] = x as u32;
        log[x % 256] = i as u32;
        x = (galois_multiply(x as u32, 2, prime)) as usize;
    }
}

// table index helper function
// NOTE: the text uses modulo 255 rather than 256, which is a complete mystery to me
// i've elected to keep going with 256 anyway, but if i encounter bugs this might be why
fn i(x: u32) -> usize {
    (x % 256) as usize
}

// "gf_mul"
pub fn table_multiply(x: u32, y: u32, tables: &([u32; 256], [u32; 256])) -> u32 {
    if x == 0 || y == 0 {
        0
    } else {
        let (exp, log) = tables;
        exp[i(log[i(x)] + log[i(y)])]
    }
}

// "gf_div"
pub fn table_divide(x: u32, y: u32, tables: &([u32; 256], [u32; 256])) -> u32 {
    if y == 0 {
        panic!();
    }

    if x == 0 {
        0
    } else {
        let (exp, log) = tables;
        // again using 256 where the text uses 255
        exp[i(log[i(x)] + (256 - log[i(y)]))]
    }
}

pub fn table_pow(x: u32, power: u32, tables: &([u32; 256], [u32; 256])) -> u32 {
    let (exp, log) = tables;
    exp[i(log[i(x)] * power)]
}

// "polynomials" section starts below
// polynomials are written in descending order:
// [a, b, c, d] = ax^3 + bx^2 + cx + d
// (i personally don't think that's a good decision, but)

pub fn polynomial_scale(poly: &Vec<u32>, x: u32, tables: &([u32; 256], [u32; 256])) -> Vec<u32> {
    let mut output: Vec<u32> = Vec::new();
    for &i in poly {
        output.push(table_multiply(i, x, tables));
    }
    output
}

pub fn gf_poly_add() {
    todo!()
}
