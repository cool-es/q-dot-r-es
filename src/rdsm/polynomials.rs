use super::{galois::*, RDSM_GENERATOR_POLYNOMIALS};

// a polynomial over a galois field, ordered from highest power of x to lowest
pub(crate) type Polynomial = Vec<Element>;

// "polynomials" section starts below
// polynomials are written in descending order:
// [a, b, c, d] = ax^3 + bx^2 + cx + d
// (i personally don't think that's a good decision, but)

pub(crate) fn polynomial_add(poly1: &Polynomial, poly2: &Polynomial) -> Polynomial {
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

// here's something i came up with...
// it was simpler in my head.
pub(crate) fn es_polynomial_multiply(poly1: &Polynomial, poly2: &Polynomial) -> Polynomial {
    let mut output: Polynomial = Vec::new();
    let (deg1, deg2) = (poly1.len() - 1, poly2.len() - 1);
    output.resize(deg1 + deg2 + 1, 0);

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
pub(crate) fn make_rdsm_generator_polynomial(ec_symbols: u32) -> Polynomial {
    // from what i can tell, the end result here is
    // (x + 1)(x + a)(x + a^2)...(x + a^ec_symbols)
    let mut output: Polynomial = vec![1];
    for i in 0..ec_symbols {
        // this value is the polynomial x + a^i ... does this actually line up
        // with the qr code standard? is a == 0000_0010 ?
        let multiplier: Polynomial = vec![1, table_pow(2, i)];
        output = es_polynomial_multiply(&output, &multiplier);
    }
    output
}

// helper function for polynomial_remainder
fn leading_zeros(poly: &Polynomial) -> usize {
    for (i, &coefficient) in poly.iter().enumerate() {
        if coefficient != 0 {
            return i;
        }
    }
    poly.len() - 1
}

pub(crate) fn polynomial_remainder(dividend: &Polynomial, divisor: &Polynomial) -> Polynomial {
    assert!(divisor[0] != 0, "divisor has a leading 0");
    let diff = if let Some(d) = dividend.len().checked_sub(divisor.len()) {
        d
    } else {
        return dividend.clone();
    };
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

    let lead = leading_zeros(&output);
    assert!(lead >= diff);

    // output starts with a bunch of 0s
    output[lead..].to_vec()
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
pub(crate) fn encode_message(message: &Polynomial, ec_symbols: u32) -> Polynomial {
    // will only generate codes "manually" if they are not qr standard
    let generator_polynomial: Polynomial =
        if let Some(index) = crate::qr_standard::find_errc(ec_symbols as usize) {
            RDSM_GENERATOR_POLYNOMIALS[index].to_vec()
        } else {
            make_rdsm_generator_polynomial(ec_symbols)
        };

    let mut message_padded = message.clone();
    message_padded.extend(std::iter::repeat(0).take(ec_symbols as usize));

    let remainder = polynomial_remainder(&message_padded, &generator_polynomial);

    polynomial_add(&message_padded, &remainder)
}
