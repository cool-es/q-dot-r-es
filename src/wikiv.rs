use core::panic;
use std::{cmp, iter};

// functions from the wikiversity "reed-solomon codes for coders" article
const DEBUG: bool = false;
// the qr-specific generator polynomial, 0b10100110111
pub const QR_GEN: u32 = 0x537;
// recurring polynomial in the wikiversity article, unsure of its significance
pub const PRIM: u32 = 0x11d;

// an element in a galois field
type element = u32;
// a polynomial over a galois field, ordered from highest power of x to lowest
type polynomial = Vec<element>;
// exp/log tables for the "table_*" functions
type exp_log_luts = ([element; 256], [element; 256]);
// lut for the reed-solomon generator polynomials
type rsgen_lut = [polynomial; 64];

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
pub fn carryless_multiply(x: element, y: element) -> element {
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

pub fn galois_multiply(x: element, y: element, prime: element) -> element {
    if prime > 0 {
        carryless_divide(carryless_multiply(x, y), prime)
    } else {
        carryless_multiply(x, y)
    }
}

pub fn qr_multiply(x: element, y: element) -> element {
    galois_multiply(x, y, QR_GEN)
}

pub fn bit_length(n: u32) -> u32 {
    match n {
        0 => 0,
        _ => n.ilog2() + 1,
    }
}

pub fn carryless_divide(dividend: element, divisor: element) -> element {
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
    x: element,
    y: element,
    prime: element,
    field_charac_full: u32,
    carryless: bool,
) -> element {
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
pub fn galois_multiply_peasant_qr(x: element, y: element) -> element {
    galois_multiply_peasant_full(x, y, QR_GEN, 256, true)
}

// NEW ADDITIONS BELOW: section "multiplication with logarithms" starts here

// "init_tables", freely interpreted
// i don't understand why this function would reach all entries in log...
pub fn generate_exp_log_tables(tables: &mut exp_log_luts, prime: element) {
    let (exp, log) = tables;
    let mut x: usize = 1;
    for i in 0..256 {
        exp[i] = x as element;
        log[x % 256] = i as element;
        x = (galois_multiply(x as element, 2, prime)) as usize;
    }
}

// table index helper function
// NOTE: the text uses modulo 255 rather than 256, which is a complete mystery to me
// i've elected to keep going with 256 anyway, but if i encounter bugs this might be why
fn i(x: u32) -> usize {
    (x % 256) as usize
}

// "gf_mul"
pub fn table_multiply(x: element, y: element, tables: &exp_log_luts) -> element {
    if x == 0 || y == 0 {
        0
    } else {
        let (exp, log) = tables;
        // exp(log(x) + log(y))
        exp[i(log[i(x)] + log[i(y)])]
    }
}

// "gf_div"
pub fn table_divide(x: element, y: element, tables: &exp_log_luts) -> element {
    if y == 0 {
        panic!();
    }

    if x == 0 {
        0
    } else {
        let (exp, log) = tables;
        // exp(log(x) - log(y))
        // again using 256 where the text uses 255
        exp[i(log[i(x)] + (256 - log[i(y)]))]
    }
}

pub fn table_pow(x: element, power: element, tables: &exp_log_luts) -> element {
    let (exp, log) = tables;
    exp[i(log[i(x)] * power)]
}

// "polynomials" section starts below
// polynomials are written in descending order:
// [a, b, c, d] = ax^3 + bx^2 + cx + d
// (i personally don't think that's a good decision, but)

pub fn polynomial_scale(poly: &polynomial, x: element, tables: &exp_log_luts) -> polynomial {
    let mut output: polynomial = Vec::new();
    for &i in poly {
        output.push(table_multiply(i, x, tables));
    }
    output
}

pub fn gf_poly_add(poly1: &polynomial, poly2: &polynomial) -> polynomial {
    let mut output: polynomial = Vec::new();
    // resize the vector to fit the higher-degree (longer) polynomial
    let (p1_len, p2_len) = (poly1.len(), poly2.len());
    let out_len = cmp::max(p1_len, p2_len);
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
pub fn gf_poly_mul(poly1: &polynomial, poly2: &polynomial, tables: &exp_log_luts) -> polynomial {
    let mut output: polynomial = Vec::new();
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
pub fn gf_poly_eval(poly: &polynomial, x: element, tables: &exp_log_luts) -> element {
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
// symbol_amnt is the number of error correcting symbols
pub fn rs_generator_poly(symbol_amnt: u32, tables: &exp_log_luts) -> polynomial {
    let mut output: polynomial = vec![1];
    for i in 0..symbol_amnt {
        let multiplier: polynomial = vec![1, table_pow(2, i, tables)];
        output = gf_poly_mul(&output, &multiplier, tables);
    }
    output
}

// adding this (not in the text) because recalculating the values over and over would be obscene
pub fn generate_rsgen_table(gentable: &mut rsgen_lut, tables: &exp_log_luts) {
    for i in 0..gentable.len() {
        gentable[i] = rs_generator_poly(i as u32, tables);
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
    #normalizer = divisor[0] # precomputing for performance
    for i in range(0, len(dividend) - (len(divisor)-1)):
        #msg_out[i] /= normalizer # for general polynomial division (when polynomials are non-monic), the usual way of using
                                  # synthetic division is to divide the divisor g(x) with its leading coefficient, but not needed here.
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
pub fn gf_poly_div(
    dividend: &polynomial,
    divisor: &polynomial,
    tables: &exp_log_luts,
) -> (polynomial, polynomial) {
    // man, idk
    todo!()
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
pub fn rs_encode_msg(msg_in: polynomial, symbol_amnt: u32, tables: &exp_log_luts) -> polynomial {
    let gen = rs_generator_poly(symbol_amnt, tables);

    // i don't know what i'm doing
    let mut output = msg_in.clone();
    output.extend(iter::repeat(0).take(gen.len() - 1));

    // i do not know what i am doing.
    let mut remainder = gf_poly_div(&output, &gen, tables).1;
    output.append(&mut remainder);
    output
}

// "Simple, isn't it?" get bent

/*
def rs_encode_msg(msg_in, nsym):
    '''Reed-Solomon main encoding function, using polynomial division (algorithm Extended Synthetic Division)'''
    if (len(msg_in) + nsym) > 255: raise ValueError("Message is too long (%i when max is 255)" % (len(msg_in)+nsym))
    gen = rs_generator_poly(nsym)
    # Init msg_out with the values inside msg_in and pad with len(gen)-1 bytes (which is the number of ecc symbols).
    msg_out = [0] * (len(msg_in) + len(gen)-1)
    # Initializing the Synthetic Division with the dividend (= input message polynomial)
    msg_out[:len(msg_in)] = msg_in

    # Synthetic division main loop
    for i in range(len(msg_in)):
        # Note that it's msg_out here, not msg_in. Thus, we reuse the updated value at each iteration
        # (this is how Synthetic Division works: instead of storing in a temporary register the intermediate values,
        # we directly commit them to the output).
        coef = msg_out[i]

        # log(0) is undefined, so we need to manually check for this case. There's no need to check
        # the divisor here because we know it can't be 0 since we generated it.
        if coef != 0:
            # in synthetic division, we always skip the first coefficient of the divisior, because it's only used to normalize the dividend coefficient (which is here useless since the divisor, the generator polynomial, is always monic)
            for j in range(1, len(gen)):
                msg_out[i+j] ^= gf_mul(gen[j], coef) # equivalent to msg_out[i+j] += gf_mul(gen[j], coef)

    # At this point, the Extended Synthetic Divison is done, msg_out contains the quotient in msg_out[:len(msg_in)]
    # and the remainder in msg_out[len(msg_in):]. Here for RS encoding, we don't need the quotient but only the remainder
    # (which represents the RS code), so we can just overwrite the quotient with the input message, so that we get
    # our complete codeword composed of the message + code.
    msg_out[:len(msg_in)] = msg_in

    return msg_out
*/
