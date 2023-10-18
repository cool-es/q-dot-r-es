use super::{galois::*, RDSM_GENERATOR_POLYNOMIALS};

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

// pub fn polynomial_scalar_multiply(poly: &Polynomial, scalar: Element) -> Polynomial {
//     let mut output: Polynomial = Vec::new();
//     for &i in poly {
//         output.push(table_multiply(i, scalar));
//     }
//     output
// }

pub fn polynomial_add(poly1: &Polynomial, poly2: &Polynomial) -> Polynomial {
    let mut output: Polynomial = Vec::new();
    // resize the vector to fit the higher-degree (longer) polynomial
    let (p1_len, p2_len) = (poly1.len(), poly2.len());
    let out_len = p1_len.max(p2_len);
    output.resize(out_len, 0);

    for i in 0..p1_len {
        output[i + out_len - p1_len] = poly1[i];
    }
    for i in 0..p2_len {
        output[i + out_len - p2_len] ^= poly2[i];
    }

    output
}

// SUPPOSEDLY multiplies two polynomials over a galois field
// need to dig into this - this seems very fishy
pub fn polynomial_multiply(poly1: &Polynomial, poly2: &Polynomial) -> Polynomial {
    let mut output: Polynomial = Vec::new();
    // since a polynomial's "length" is its degree + 1,
    // output is degree d1+d2 => length (d1+1)+(d2+1)-1
    // len() and resize() both count from 1, no fencepost error
    output.resize(poly1.len() + poly2.len() - 1, 0);

    // ... i+j won't reach the top coefficient?
    // yes it will, it goes up to (l1+l2-1)-1
    for i in 0..poly2.len() {
        for j in 0..poly1.len() {
            // the tutorial claims that this line is:
            // "equivalent to r[i + j] = gf_add(r[i+j], gf_mul(p[i], q[j]))"
            output[i + j] ^= table_multiply(poly1[j], poly2[i]);
        }
    }
    output
}

// here's something i came up with...
// it was simpler in my head.
pub fn es_polynomial_multiply(poly1: &Polynomial, poly2: &Polynomial) -> Polynomial {
    let mut output: Polynomial = Vec::new();
    let (deg1, deg2) = (poly1.len() - 1, poly2.len() - 1);
    output.resize(deg1 + deg2 + 1, 0);

    // for each degree value
    /*     for deg_step in 0..(deg1 + deg2) {
        let mut sum = 0;

        // i + j = k

        for i in 0..=deg_step {
            let j = deg_step - i;
            if i > deg1 || j > deg2 {
                continue;
            }
            sum ^= table_multiply(poly1[i], poly2[j]);
        }
        output[deg_step] = sum;
    } */

    // imagine we write the polynomial product as a rectangle -
    // then, all the coefficients of the same degree lie along diagonals.
    // we sum all these coefficients before writing them to the output polynomial
    for horiz_step in 0..deg1 {
        let mut sum = 0;

        for i in (0..=horiz_step).rev() {
            let j = horiz_step - i;
            if j > deg2 {
                // out of bounds
                break;
            }
            sum ^= table_multiply(poly1[i], poly2[j]);
        }
        output[horiz_step] = sum;
    }

    for vert_step in 0..=deg2 {
        let mut sum = 0;

        for j in vert_step..=(vert_step + deg1) {
            let i = (vert_step + deg1) - j;
            if j > deg2 {
                // out of bounds
                break;
            }
            sum ^= table_multiply(poly1[i], poly2[j]);
        }
        output[vert_step + deg1] = sum;
    }

    output
}

// evaluates a polynomial for a specific value of x
// "based on horner's scheme for maximum efficiency"
/* pub fn polynomial_evaluate(poly: &Polynomial, x: Element, tables: &ExpLogLUTs) -> Element {
    let mut output = poly[0];
    for i in 1..poly.len() {
        output = table_multiply(output, x) ^ poly[i];
    }
    output
}
 */
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
pub fn make_rdsm_generator_polynomial(ec_symbols: u32) -> Polynomial {
    // from what i can tell, the end result here is
    // (x + 1)(x + a)(x + a^2)...(x + a^ec_symbols)
    let mut output: Polynomial = vec![1];
    for i in 0..ec_symbols {
        // this value is the polynomial x + a^i ... does this actually line up
        // with the qr code standard? is a == 0000_0010 ?
        let multiplier: Polynomial = vec![1, table_pow(2, i)];
        output = polynomial_multiply(&output, &multiplier);
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
// pub fn _polynomial_divide(
//     dividend: &Polynomial,
//     divisor: &Polynomial,
//     // tables: &ExpLogLUTs,
// ) -> (Polynomial, Polynomial) {
//     // man, idk

//     let mut output = dividend.clone();
//     // output.reverse();
//     // let mut dnd = dividend.clone();
//     // dnd.reverse();
//     // let mut dsr = divisor.clone();
//     // dsr.reverse();

//     for i in 0..(dividend.len() - (divisor.len() - 1)) {
//         let coef = output[i];
//         if coef != 0 {
//             for j in 1..divisor.len() {
//                 if divisor[j] != 0 {
//                     output[i + j] ^= table_multiply(divisor[j], coef);
//                 }
//             }
//         }
//     }

//     // output.reverse();
//     let (quotient, remainder) = output.split_at(divisor.len() - 1);
//     // let (quotient, remainder) = output.split_at(divisor.len() - 1);
//     (quotient.to_vec(), remainder.to_vec())
// }

// helper function
pub fn length(poly: &Polynomial) -> usize {
    poly.len() - leading_zeroes(poly)
}

// helper function for polynomial_remainder
fn leading_zeroes(poly: &Polynomial) -> usize {
    for (i, &coefficient) in poly.iter().enumerate() {
        if coefficient != 0 {
            return i;
        }
    }
    poly.len() - 1
}

pub fn polynomial_remainder(dividend: &Polynomial, divisor: &Polynomial) -> Polynomial {
    if divisor[0] == 0 {
        panic!()
    }
    if dividend.len() < divisor.len() {
        return dividend.clone();
    }
    let diff = dividend.len() - divisor.len();
    let mut output = dividend.clone();
    // rightwards index shift in output (equivalent to multiplying divisor by x^(diff-shift))
    // if you wanted the quotient too, it would be {q[shift] = multiplier}, then reverse q
    for shift in 0..=diff {
        if output[shift] == 0 {
            continue;
        }
        let multiplier = table_divide(output[shift], divisor[0]);

        for index in 0..divisor.len() {
            output[index + shift] ^= table_multiply(divisor[index], multiplier);
        }
    }

    if leading_zeroes(&output) < diff {
        panic!()
    }
    // output starts with a bunch of 0s
    output[leading_zeroes(&output)..].to_vec()
}

// it works!!! i'm doing encodation!!!!!
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
pub fn encode_message(message: &Polynomial, ec_symbols: u32) -> Polynomial {
    // will only generate codes "manually" if they are not qr standard
    let generator_polynomial: Polynomial = {
        if let Some(index) = crate::qr_standard::find_errc(ec_symbols as usize) {
            RDSM_GENERATOR_POLYNOMIALS[index].to_vec()
        } else {
            make_rdsm_generator_polynomial(ec_symbols)
        }
    };

    let mut message_padded = message.clone();
    message_padded.extend(std::iter::repeat(0).take(ec_symbols as usize));

    let remainder = polynomial_remainder(&message_padded, &generator_polynomial);
    let output = polynomial_add(&message_padded, &remainder);

    output
}

pub fn charprint(poly: &Polynomial) {
    let mut output = String::new();
    for &i in poly {
        let o = i as u8;
        output.push({
            if o.is_ascii_control() {
                '😨'
            } else {
                o as char
            }
        });
    }
    println!("{:?}", output);
}

pub fn prettyprint(poly: &Polynomial) {
    fn superscript(input: usize) -> String {
        // ¹²³⁴⁵⁶⁷⁸⁹⁰
        let mut output = String::new();
        if input == 0 {
            output.push_str("ˣ");
            return output;
        } else if input == 1 {
            return output;
        }
        for i in (0..=input.ilog10()).rev() {
            let digit = (input as u32 / 10u32.pow(i as u32)) % 10;
            output.push(match digit {
                1 => '¹',
                2 => '²',
                3 => '³',
                _ => char::from_u32('⁰' as u32 + digit).unwrap(),
            })
        }
        output
    }

    let mut output = String::new();
    let last_byte_not_zero = *poly.last().unwrap() != 0;
    if poly[0] == 0 {
        // polynomial has leading zeroes - it shouldn't
        output.push('🤔');
    }
    for i in 0..poly.len() {
        if poly[i] != 0 {
            let mut part: String;
            if poly[i] == 1 {
                part = format!("x{}", superscript(poly.len() - (i + 1)));
            } else if i == poly.len() - 1 {
                part = format!("a{}", superscript(log(poly[i])),);
            } else {
                part = format!(
                    "a{}x{}",
                    superscript(log(poly[i])),
                    superscript(poly.len() - (i + 1))
                );
            }
            if i < poly.len() - 1 && last_byte_not_zero {
                part.push_str(" + ");
            }
            output.push_str(part.as_str());
        }
    }
    println!("{}", output);
}

pub fn split_to_blocks_and_encode(
    polynomial: &Polynomial,
    info: crate::qr_standard::VersionBlockInfo,
) -> Vec<Polynomial> {
    // number of blocks of this type, codewords per block, data codewords per block
    // note that the number of error correcting codewords is the same for all blocks!
    let (bc, cw, dcw, optional) = info;
    // let (bc2, _, dcw2) = optional.unwrap_or((0, 0, 1));

    if optional.is_some() {
        panic!("multiple block types are not supported yet")
    }
    let (bc2, dcw2) = (0, 0);

    // check to make sure poly will split evenly
    assert!(
        polynomial.len() == dcw * bc + dcw2 * bc2,
        "could not split to blocks - stream is {} codewords but alotted space is {}",
        polynomial.len(),
        dcw * bc + dcw2 * bc2
    );

    let mut unencoded: Vec<Polynomial> = Vec::new();

    for i in 0..bc {
        let (a, b) = (i * dcw, (i + 1) * dcw);
        unencoded.push(polynomial[a..b].to_vec());
    }

    let mut output = Vec::new();

    for i in unencoded {
        output.push(encode_message(&i, (cw - dcw) as u32));
    }

    output
}
