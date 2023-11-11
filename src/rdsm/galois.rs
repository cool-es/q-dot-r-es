use crate::*;
// functions from the wikiversity "reed-solomon codes for coders" article

// an element in the finite field GF(2^8)
pub(crate) type Element = u8;
// a polynomial over GF(2)
pub(crate) type BigElement = u32;

// qr data generator/divisor polynomial, 0b100011101
#[allow(dead_code)]
pub(crate) const QR_CODEWORD_GEN: BigElement = 0x11D;
// qr format generator/divisor polynomial, 0b10100110111
pub(crate) const QR_FORMAT_GEN: BigElement = 0x537;

// exp/log tables for the "table_*" functions
pub(super) const EXPVALUES: usize = 255;
pub(super) const LOGVALUES: usize = EXPVALUES;
pub(crate) type ExpLogLUTs = ([Element; EXPVALUES], [usize; LOGVALUES]);

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
pub(crate) fn qr_fcode_remainder(fcode: u32) -> u32 {
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

// (fmt * 2^10 + remainder of (fmt * 2^10) / g) - this always has remainder 0
// this works since all numbers in a galois field are their own additive inverse,
// and since (remainder of (k + remainder of k)) == (remainder of k + remainder of k)
pub(crate) fn qr_generate_fcode(fmt: u8) -> Option<u16> {
    if fmt >= 32 {
        return None;
    }

    // i'm aware that this code is ridiculous
    Some(((fmt as u16) << 10) | (qr_fcode_remainder((fmt as u32) << 10)) as u16)
}

#[inline]
pub(crate) fn bit_length(n: BigElement) -> u32 {
    if let Some(x) = n.checked_ilog2() {
        x + 1
    } else {
        0
    }
}

pub(crate) fn carryless_divide(dividend: BigElement, divisor: BigElement) -> BigElement {
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
        if dnd & (1 << (i + dsr_length - 1)) != 0 {
            dnd ^= divisor << i;
        }
    }
    dnd
}

// helper function, uses precomputed tables
#[inline]
pub(crate) fn exp(n: usize) -> Element {
    QR_EXP_LOG_TABLE.0[n % 255]
}

// ditto
// i keep debating whether to make this function ->  Option<usize> instead
// it would make the other functions a lot uglier though!
pub(crate) fn log(e: Element) -> usize {
    if e == 0 {
        panic!()
    } else {
        QR_EXP_LOG_TABLE.1[((e - 1) % 255) as usize]
    }
}

// "gf_mul"
pub(crate) fn table_multiply(x: Element, y: Element) -> Element {
    if x == 0 || y == 0 {
        return 0;
    }

    exp(log(x) + log(y))
}

// "gf_div"
pub(crate) fn table_divide(x: Element, y: Element) -> Element {
    if y == 0 {
        panic!()
    } else if x == 0 {
        return 0;
    }

    exp(log(x) + (255 - log(y)))
}

pub(crate) fn table_pow(x: Element, power: u32) -> Element {
    exp(log(x) * power as usize)
}
