use crate::*;
// functions from the wikiversity "reed-solomon codes for coders" article

// qr data generator/divisor polynomial, 0b100011101
pub const QR_CODEWORD_GEN: Element = 0x11D;
// qr format generator/divisor polynomial, 0b10100110111
pub const QR_FORMAT_GEN: Element = 0x537;

// an element in the finite field GF(2^8)
pub type Element = u32;

// exp/log tables for the "table_*" functions
pub(super) const EXPVALUES: usize = 255;
pub(super) const LOGVALUES: usize = 255;
pub type ExpLogLUTs = ([Element; EXPVALUES], [usize; LOGVALUES]);

// blank tables to make initialization easier
pub const BLANK_EXP_LOG_LUTS: ExpLogLUTs = ([0; EXPVALUES], [0; LOGVALUES]);

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
pub fn qr_fcode_remainder(fcode: u32) -> u32 {
    // 0b10100110111
    let mut output = fcode;

    for i in (0..=4).rev() {
        if (1 << (i + 10)) & output != 0 {
            // the 2^(i+10) bit of fmt is 1

            // add (without carry) g shifted by i
            // because g has a 1 in the highest, 2^10, bit,
            // this will always erase that bit of fmt.
            // in essence we're doing like, "lights out"
            // on the 2^14 to 2^10 bits, from high to low
            output ^= QR_FORMAT_GEN << i;
        }
    }
    output
}

pub fn qr_fcode_is_good(fcode: u16) -> bool {
    qr_fcode_remainder(fcode as u32) == 0
}

// (fmt * 2^10 + remainder of (fmt * 2^10) / g) - this always has remainder 0
// this works since all numbers in a galois field are their own additive inverse,
// and since (remainder of (k + remainder of k)) == (remainder of k + remainder of k)
pub fn qr_generate_fcode(fmt: u8) -> Option<u16> {
    if fmt >= 32 {
        return None;
    }

    // i'm aware that this code is ridiculous
    Some(((fmt as u16) << 10) | (qr_fcode_remainder((fmt as u32) << 10)) as u16)
}

// earlier "qr_decode_format"
pub fn qr_find_fmt(fcode: u16) -> Option<u8> {
    // looks complex, is actually very simple:
    // try every format, generate its format code,
    // check the format code against the input
    // (lowest difference wins). returns None
    // if there's a tie
    let mut best_format: Option<u8> = None;
    let mut best_distance = 15;

    for try_format in 0..32 {
        let try_fcode = qr_generate_fcode(try_format)?;
        let try_distance = (fcode ^ try_fcode).count_ones();

        if try_distance < best_distance {
            best_distance = try_distance;
            best_format = Some(try_format);
        } else if try_distance == best_distance {
            best_format = None;
        }
    }
    best_format
}

// carry-less multiplication ("cl_mul")
// this is used for multiplying GF(2^8) elements,
// that is, the individual codewords in a qr symbol
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

// carry-less multiplication, modulo a primitive irreducible polynomial
// this always forms a cyclic group, and under the qr generator polynomial,
// that group happens to be generated by the polynomial 1x + 0 (afaict)
pub fn galois_multiply(x: Element, y: Element, primitive: Element) -> Element {
    if primitive > 0 {
        carryless_divide(carryless_multiply(x, y), primitive)
    } else {
        carryless_multiply(x, y)
    }
}

#[inline]
pub fn bit_length(n: Element) -> u32 {
    if let Some(x) = n.checked_ilog2() {
        x + 1
    } else {
        0
    }
}

pub fn carryless_divide(dividend: Element, divisor: Element) -> Element {
    if bit_length(dividend) < bit_length(divisor) {
        return dividend;
    }

    let dnd_length = bit_length(dividend);
    let dsr_length = bit_length(divisor);
    let mut dnd = dividend;

    // long division: for each bit that separates the dividend and divisor,
    // subtract (i.e. add, in GF(2^n)) the divisor shifted up so its top bit
    // lines up with the dividend's current top bit. it's "lights out" again!
    for i in (0..=(dnd_length - dsr_length)).rev() {
        if dnd & (1 << i + dsr_length - 1) != 0 {
            dnd ^= divisor << i;
        }
    }
    dnd
}

// NEW ADDITIONS BELOW: section "multiplication with logarithms" starts here

// "init_tables", freely interpreted
// i don't understand why this function would reach all entries in log...
// answer: it won't, necessarily! it only does given the right primitive polynomial,
// so therefore i've taken the liberty of hard-coding in the polynomial to use.
// this function is also a bit cryptic - it's based on that the multiplicative
// group in a finite field is always cyclic, and that arithmetic modulo a polynomial
// is homomorphic to general polynomial arithmetic. therefore, you will reach every
// possible remainder this way, and they will replace the full-size polynomials just fine
pub fn generate_exp_log_tables(tables: &mut ExpLogLUTs) {
    let (exp, log) = tables;
    let mut x: Element = 1;

    // the intention here is that exp[i] == a^i mod p, so this should not
    // assign a value to log[0]... if it does, something is wrong
    // note that exp is a function N -> GF(2^8), and log is GF(2^8) -> N !
    // so therefore, the log table has 255 usize values.
    // log table is mod 255 because x^(q-1) = 1 for all elements in GF(q) except 0
    for i in 0..255 {
        exp[i] = x;
        // log(x) == log[x - 1]
        log[(x as usize - 1) % 255] = i as usize;
        // note that the logarithm operation is base 0b10
        x = galois_multiply(x, 0b10, QR_CODEWORD_GEN);
    }
}

// helper function, uses precomputed tables
#[inline]
pub fn exp(n: usize) -> Element {
    QR_EXP_LOG_TABLE.0[n % 255]
}

// ditto
// i keep debating whether to make this function ->  Option<usize> instead
// it would make the other functions a lot uglier though!
pub fn log(e: Element) -> usize {
    if e == 0 {
        panic!()
    } else {
        QR_EXP_LOG_TABLE.1[((e - 1) % 255) as usize]
    }
}

// "gf_mul"
pub fn table_multiply(x: Element, y: Element) -> Element {
    if x == 0 || y == 0 {
        return 0;
    }

    exp(log(x) + log(y))
}

// "gf_div"
pub fn table_divide(x: Element, y: Element) -> Element {
    if y == 0 {
        panic!()
    } else if x == 0 {
        return 0;
    }

    exp(log(x) + (255 - log(y)))
}

pub fn table_pow(x: Element, power: u32) -> Element {
    exp(log(x) * power as usize)
}

// old versions of the table operations
// not fully tested or confirmed to work flawlessly
mod _old {
    use super::*;

    // table index helper function
    fn element_to_usize(e: Element) -> usize {
        if e == 0 {
            panic!()
        } else {
            ((e - 1) % 255) as usize
        }
    }

    // "gf_mul"
    pub fn table_multiply(x: Element, y: Element, tables: &ExpLogLUTs) -> Element {
        if x == 0 || y == 0 {
            return 0;
        }

        let (exp, log) = tables;
        // exp(log(x) + log(y))
        exp[(log[element_to_usize(x)] + log[element_to_usize(y)]) % 255]
    }

    // "gf_div"
    pub fn table_divide(x: Element, y: Element, tables: &ExpLogLUTs) -> Element {
        if y == 0 {
            panic!()
        }
        if x == 0 {
            return 0;
        }

        let (exp, log) = tables;
        // exp(log(x) - log(y))
        exp[(log[element_to_usize(x)] + (255 - log[element_to_usize(y)])) % 255]
    }

    pub fn table_pow(x: Element, power: u32, tables: &ExpLogLUTs) -> Element {
        let (exp, log) = tables;
        exp[((log[element_to_usize(x)]) * power as usize) % 255]
    }

    // uses russian peasant multiplication
    // default values prim = 0 field_charac_full = 256, carryless = true
    /*
        Galois Field integer multiplication using Russian Peasant Multiplication algorithm
        (faster than the standard multiplication + modular reduction).
        If prim is 0 and carryless=False, then the function produces the result for
        a standard integer multiplication (no carry-less arithmetics nor modular reduction).
    */
    // i see literally no reason to use this
    /* pub fn galois_multiply_peasant_full(
        x: Element,
        y: Element,
        primitive: Element,
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
            if primitive > 0 && x & (field_charac_full as Element) != 0 {
                x = x ^ primitive;
            }
        }
        output
    } */

    // attempting to make a nicer peasant multiply...
    // not sure what the field character is supposed to be, but i'm guessing 256
    // not using this
    /* pub fn galois_multiply_peasant_qr(x: Element, y: Element) -> Element {
        galois_multiply_peasant_full(x, y, QR_CODEWORD_GEN, 256, true)
    }
     */
}
