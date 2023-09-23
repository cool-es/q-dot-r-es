
// functions from the wikiversity "reed-solomon codes for coders" article
// the qr-specific generator polynomial, 0b10100110111
pub const QR_GEN: u32 = 0x537;
// recurring polynomial in the wikiversity article, unsure of its significance
pub const PRIM: u32 = 0x11d;

// from the tutorial: uses PRIM as its generator polynomial
// rs_encode_msg(TEST_MSG, 10) == TEST_MSG + TEST_RESULT == FULL_TEST_RESULT
// length 16
pub const TEST_MSG: &[Element] = &[
    0x40, 0xd2, 0x75, 0x47, 0x76, 0x17, 0x32, 0x06, 0x27, 0x26, 0x96, 0xc6, 0xc6, 0x96, 0x70, 0xec,
];
// length 10
pub const TEST_RESULT: &[Element] = &[0xbc, 0x2a, 0x90, 0x13, 0x6b, 0xaf, 0xef, 0xfd, 0x4b, 0xe0];
// length 26
pub const FULL_TEST_RESULT: &[Element] = &[
    0x40, 0xd2, 0x75, 0x47, 0x76, 0x17, 0x32, 0x06, 0x27, 0x26, 0x96, 0xc6, 0xc6, 0x96, 0x70, 0xec,
    0xbc, 0x2a, 0x90, 0x13, 0x6b, 0xaf, 0xef, 0xfd, 0x4b, 0xe0,
];

// an element in a galois field
pub type Element = u32;
// a polynomial over a galois field, ordered from highest power of x to lowest
pub type Polynomial = Vec<Element>;
// exp/log tables for the "table_*" functions
pub type ExpLogLUTs = ([Element; 512], [Element; 256]);
// lut for the reed-solomon generator polynomials (not fit for use atm)
pub type _RSGenLUT = [Polynomial; 64];

// blank tables to make initialization easier
pub const BLANK_EXP_LOG_LUTS: ExpLogLUTs = ([0; 512], [0; 256]);

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

    for i in (0..=4).rev() {
        if (1 << (i + 10)) & output != 0 {
            // the 2^(i+10) bit of fmt is 1

            // add (without carry) g shifted by i
            // because g has a 1 in the highest, 2^10, bit,
            // this will always erase that bit of fmt.
            // in essence we're doing like, "lights out"
            // on the 2^14 to 2^10 bits, from high to low
            output ^= qr_gen << i;
        }
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
        core::panic!();
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
pub fn carryless_multiply(x: Element, y: Element) -> Element {
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

pub fn galois_multiply(x: Element, y: Element, prime: Element) -> Element {
    if prime > 0 {
        carryless_divide(carryless_multiply(x, y), prime)
    } else {
        carryless_multiply(x, y)
    }
}

pub fn qr_multiply(x: Element, y: Element) -> Element {
    galois_multiply(x, y, QR_GEN)
}

pub fn bit_length(n: u32) -> u32 {
    match n {
        0 => 0,
        _ => n.ilog2() + 1,
    }
}

pub fn carryless_divide(dividend: Element, divisor: Element) -> Element {
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
    x: Element,
    y: Element,
    prime: Element,
    field_charac_full: u32,
    carryless: bool,
) -> Element {
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
pub fn galois_multiply_peasant_qr(x: Element, y: Element) -> Element {
    galois_multiply_peasant_full(x, y, QR_GEN, 256, true)
}

// NEW ADDITIONS BELOW: section "multiplication with logarithms" starts here

// "init_tables", freely interpreted
// i don't understand why this function would reach all entries in log...
pub fn generate_exp_log_tables(tables: &mut ExpLogLUTs, prime: Element) {
    let (exp, log) = tables;
    let mut x: usize = 1;
    for i in 0..255 {
        exp[i] = x as Element;
        log[x] = i as Element;
        x = (galois_multiply(x as Element, 2, prime)) as usize;
    }
}

// table index helper function
// NOTE: the text uses modulo 255 rather than 256, which is a complete mystery to me
// i've elected to keep going with 256 anyway, but if i encounter bugs this might be why
fn i(x: u32) -> usize {
    //(x % 256) as usize
    (x % 255) as usize
}

// "gf_mul"
pub fn table_multiply(x: Element, y: Element, tables: &ExpLogLUTs) -> Element {
    if x == 0 || y == 0 {
        0
    } else {
        let (exp, log) = tables;
        // exp(log(x) + log(y))
        exp[i(log[i(x)] + log[i(y)])]
    }
}

//fake function, debug: has the same signature as table_multiply but doesn't use tables
pub fn ftable_multiply(x: Element, y: Element, _tables: &ExpLogLUTs) -> Element {
    galois_multiply(x, y, PRIM)
}

// "gf_div"
pub fn table_divide(x: Element, y: Element, tables: &ExpLogLUTs) -> Element {
    if y == 0 {
        core::panic!();
    }

    if x == 0 {
        0
    } else {
        let (exp, log) = tables;
        // exp(log(x) - log(y))
        // again using 256 where the text uses 255
        // exp[i(log[i(x)] + (256 - log[i(y)]))]
        exp[i(log[i(x)] + (255 - log[i(y)]))]
    }
}

pub fn table_pow(x: Element, power: u32, tables: &ExpLogLUTs) -> Element {
    let (exp, log) = tables;
    exp[i(log[i(x)] * power)]
}

// another fake "table" debug function
pub fn ftable_pow(x: Element, power: u32, _tables: &ExpLogLUTs) -> Element {
    let mut output = 1;
    for i in 0..power {
        output = galois_multiply(output, x, PRIM);
    }
    output
}
