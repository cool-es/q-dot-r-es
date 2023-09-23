// reed-solomon / galois field operations from wikiversity:
// https://en.wikiversity.org/wiki/Reed%E2%80%93Solomon_codes_for_coders


use core::panic;


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
pub type ExpLogLUTs = ([Element; 256], [Element; 256]);
// lut for the reed-solomon generator polynomials (not fit for use atm)
pub type _RSGenLUT = [Polynomial; 64];

// blank tables to make initialization easier
pub const BLANK_EXP_LOG_LUTS: ExpLogLUTs = ([0; 256], [0; 256]);

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
    for i in 0..256 {
        exp[i] = x as Element;
        log[x % 256] = i as Element;
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
        panic!();
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

// "polynomials" section starts below
// polynomials are written in descending order:
// [a, b, c, d] = ax^3 + bx^2 + cx + d
// (i personally don't think that's a good decision, but)

pub fn polynomial_scale(poly: &Polynomial, x: Element, tables: &ExpLogLUTs) -> Polynomial {
    let mut output: Polynomial = Vec::new();
    for &i in poly {
        output.push(table_multiply(i, x, tables));
    }
    output
}

pub fn polynomial_add(poly1: &Polynomial, poly2: &Polynomial) -> Polynomial {
    let mut output: Polynomial = Vec::new();
    // resize the vector to fit the higher-degree (longer) polynomial
    let (p1_len, p2_len) = (poly1.len(), poly2.len());
    let out_len = std::cmp::max(p1_len, p2_len);
    output.resize(out_len + 1, 0);

    for i in 0..p1_len {
        output[i + out_len - p1_len] = poly1[i];
    }
    for i in 0..p2_len {
        output[i + out_len - p2_len] ^= poly2[i];
    }

    output
}

// SUPPOSEDLY multiplies two polynomials over a galois field
pub fn polynomial_multiply(
    poly1: &Polynomial,
    poly2: &Polynomial,
    tables: &ExpLogLUTs,
) -> Polynomial {
    let mut output: Polynomial = Vec::new();
    // out_len needs to be p1_len + p2_len - 1
    // len() and resize() both count from 1, no fencepost error
    output.resize(poly1.len() + poly2.len() - 1, 0);

    for i in 0..poly2.len() {
        for j in 0..poly1.len() {
            // the tutorial claims that this line is:
            // "equivalent to r[i + j] = gf_add(r[i+j], gf_mul(p[i], q[j]))"
            output[i + j] ^= table_multiply(poly1[j], poly2[i], tables);
        }
    }
    output
}

// evaluates a polynomial for a specific value of x
// "based on horner's scheme for maximum efficiency"
pub fn polynomial_evaluate(poly: &Polynomial, x: Element, tables: &ExpLogLUTs) -> Element {
    let mut output = poly[0];
    for i in 1..poly.len() {
        output = table_multiply(output, x, tables) ^ poly[i];
    }
    output
}

// wow! this sucks!
/*
    def rs_generator_poly(nsym):
        '''Generate an irreducible generator polynomial (necessary to encode a message into Reed-Solomon)'''
        g = [1]
        for i in range(0, nsym):
            g = gf_poly_mul(g, [1, gf_pow(2, i)])
        return g
*/
// ec_symbols is the number of error correcting symbols
pub fn make_rdsm_generator_polynomial(ec_symbols: u32, tables: &ExpLogLUTs) -> Polynomial {
    let mut output: Polynomial = vec![1];
    for i in 0..ec_symbols {
        let multiplier: Polynomial = vec![1, table_pow(2, i, tables)];
        output = polynomial_multiply(&output, &multiplier, tables);
    }
    output
}

// adding this (not in the text) because recalculating the values over and over would be obscene
// lacks implementations and any practical use
pub fn _generate_rsgen_table(gentable: &mut _RSGenLUT, tables: &ExpLogLUTs) {
    for i in 0..gentable.len() {
        gentable[i] = make_rdsm_generator_polynomial(i as u32, tables);
    }
}

// help!!
/*
def gf_poly_div(dividend, divisor):
    '''Fast polynomial division by using Extended Synthetic Division and optimized for GF(2^p) computations
    (doesn't work with standard polynomials outside of this galois field, see the Wikipedia article for generic algorithm).'''
    # CAUTION: this function expects polynomials to follow the opposite convention at decoding:
    # the terms must go from the biggest to lowest degree (while most other functions here expect
    # a list from lowest to biggest degree). eg: 1 + 2x + 5x^2 = [5, 2, 1], NOT [1, 2, 5]

    msg_out = list(dividend) # Copy the dividend

    for i in range(0, len(dividend) - (len(divisor)-1)):
        coef = msg_out[i] # precaching
        if coef != 0: # log(0) is undefined, so we need to avoid that case explicitly (and it's also a good optimization).
            for j in range(1, len(divisor)): # in synthetic division, we always skip the first coefficient of the divisior,
                                              # because it's only used to normalize the dividend coefficient
                if divisor[j] != 0: # log(0) is undefined
                    msg_out[i + j] ^= gf_mul(divisor[j], coef) # equivalent to the more mathematically correct
                                                               # (but xoring directly is faster): msg_out[i + j] += -divisor[j] * coef

    # The resulting msg_out contains both the quotient and the remainder, the remainder being the size of the divisor
    # (the remainder has necessarily the same degree as the divisor -- not length but degree == length-1 -- since it's
    # what we couldn't divide from the dividend), so we compute the index where this separation is, and return the quotient and remainder.
    separator = -(len(divisor)-1)
    return msg_out[:separator], msg_out[separator:] # return quotient, remainder.
*/
pub fn polynomial_divide(
    dividend: &Polynomial,
    divisor: &Polynomial,
    tables: &ExpLogLUTs,
) -> (Polynomial, Polynomial) {
    // man, idk

    let mut output = dividend.clone();
    for i in 0..(dividend.len() - (divisor.len() - 1)) {
        let coef = output[i];
        if coef != 0 {
            for j in 1..divisor.len() {
                if divisor[j] != 0 {
                    output[i + j] ^= table_multiply(divisor[j], coef, tables);
                }
            }
        }
    }

    let (quotient, remainder) = output.split_at(divisor.len() - 1);
    (quotient.to_vec(), remainder.to_vec())
}

/*
def rs_encode_msg(msg_in, nsym):
    '''Reed-Solomon main encoding function'''
    gen = rs_generator_poly(nsym)

    # Pad the message, then divide it by the irreducible generator polynomial
    _, remainder = gf_poly_div(msg_in + [0] * (len(gen)-1), gen)
    # The remainder is our RS code! Just append it to our original message to get our full codeword (this represents a polynomial of max 256 terms)
    msg_out = msg_in + remainder
    # Return the codeword
    return msg_out
*/
pub fn encode_message(msg_in: &Polynomial, ec_symbols: u32, tables: &ExpLogLUTs) -> Polynomial {
    let gen = make_rdsm_generator_polynomial(ec_symbols, tables);

    // i don't know what i'm doing
    let mut msg_in_padded = msg_in.clone();
    msg_in_padded.extend(std::iter::repeat(0).take(gen.len() - 1));

    // i do not know what i am doing.
    let remainder = polynomial_divide(&msg_in_padded, &gen, tables).1;
    let mut output = msg_in.clone();
    output.extend(remainder.iter());
    output
}

// "Simple, isn't it?" get bent

// in theory i should have the full capability to create a qr code now
// nope it don't work

pub fn debug_check_tables(tables: &ExpLogLUTs) {}