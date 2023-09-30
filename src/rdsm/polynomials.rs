use super::galois::*;

// a polynomial over a galois field, ordered from highest power of x to lowest
pub type Polynomial = Vec<Element>;
// lut for the reed-solomon generator polynomials (not fit for use at this time)
// pub type _RSGenLUT = [Polynomial; 64];

// from the tutorial: uses QR_CODEWORD_GEN as its generator polynomial
// encode_message(TEST_MSG, 10) == TEST_MSG + TEST_RESULT == FULL_TEST_RESULT
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

// "polynomials" section starts below
// polynomials are written in descending order:
// [a, b, c, d] = ax^3 + bx^2 + cx + d
// (i personally don't think that's a good decision, but)

pub fn polynomial_scalar_multiply(
    poly: &Polynomial,
    scalar: Element,
    tables: &ExpLogLUTs,
) -> Polynomial {
    let mut output: Polynomial = Vec::new();
    for &i in poly {
        output.push(table_multiply(i, scalar));
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
            output[i + j] ^= table_multiply(poly1[j], poly2[i]);
        }
    }
    output
}

// evaluates a polynomial for a specific value of x
// "based on horner's scheme for maximum efficiency"
pub fn polynomial_evaluate(poly: &Polynomial, x: Element, tables: &ExpLogLUTs) -> Element {
    let mut output = poly[0];
    for i in 1..poly.len() {
        output = table_multiply(output, x) ^ poly[i];
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
    // from what i can tell, the end result here is
    // (x + 1)(x + a)(x + a^2)...(x + a^ec_symbols)
    let mut output: Polynomial = vec![1];
    for i in 0..ec_symbols {
        // this value is the polynomial x + a^i ... does this actually line up
        // with the qr code standard? is a == 0000_0010 ?
        let multiplier: Polynomial = vec![1, table_pow(2, i)];
        output = polynomial_multiply(&output, &multiplier, tables);
    }
    output
}

// adding this (not in the text) because recalculating the values over and over would be obscene
// lacks implementations and any practical use
// pub fn _generate_rsgen_table(gentable: &mut _RSGenLUT, tables: &ExpLogLUTs) {
//     for i in 0..gentable.len() {
//         gentable[i] = make_rdsm_generator_polynomial(i as u32, tables);
//     }
// }

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
                    output[i + j] ^= table_multiply(divisor[j], coef);
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
pub fn encode_message(message: &Polynomial, ec_symbols: u32, tables: &ExpLogLUTs) -> Polynomial {
    let generator_polynomial = make_rdsm_generator_polynomial(ec_symbols, tables);

    // i don't know what i'm doing
    let mut message_padded = message.clone();
    message_padded.extend(std::iter::repeat(0).take(generator_polynomial.len() - 1));

    // i do not know what i am doing.
    let remainder = polynomial_divide(&message_padded, &generator_polynomial, tables).1;
    let mut output = message.clone();
    output.extend(remainder.iter());
    output
}

// "Simple, isn't it?" get bent

// in theory i should have the full capability to create a qr code now
// nope it don't work
