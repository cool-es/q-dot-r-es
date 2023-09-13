// functions from the wikiversity "reed-solomon codes for coders" article

// input/output types are guesses
// returns the remainder of {fmt} divided by {g} in GF(2^8)
// named "check format" because it returns qr format data:
/*
    The format code should produce a remainder of zero 
    when it is divided by the generator of the code.
    This function can also be used to encode the 5-bit format information.
    encoded_format = (format<<10) + qr_check_format(format<<10)
*/
pub fn qr_check_format (fmt: u32) -> u32 {
    let g = 0b10100110111;
    let mut fmt = fmt;

    for i in (0..=4).rev() {
        if 0 != (fmt & (1 << (i + 10))) {
            fmt ^= g << i;
        }
    }
    return fmt;
}

// counts number of bits in x
pub fn hamming_weight (x: u32) -> u32 {
/*
    let mut x = x;
    let mut weight = 0;
    while x>0 {
        weight += x & 1;
        x >>= 1;
    }
    return weight;
*/
    // work smart not hard
    return x.count_ones();
}

pub fn qr_decode_format (fmt: u32) -> Option<u32> {
    let mut best_fmt: Option<u32> = None;
    let mut best_dist = 15;
    for test_fmt in 0..32 {
        let test_code = (test_fmt << 10) ^ qr_check_format(test_fmt<<10);
        let test_dist = hamming_weight(fmt ^ test_code);
        if test_dist < best_dist {
            best_dist = test_dist;
            best_fmt = Some(test_fmt);
        } else if test_dist == best_dist {
            best_fmt = None;
        }
    }
    return best_fmt;
}

// carry-less multiplication in GF(2^8)
pub fn cl_mul (x: u32, y: u32) -> u32 {
    let mut z = 0;
    for i in 0..=(32-y.leading_zeros()) {
        // for every 1 bit in y, xor in a copy of x
        // note that "== 1" won't work here -
        // it can be any power of 2
        if (y & (1<<i)) != 0 {
            z ^= x << i;
        }
    }
    return z;
}

pub fn gf_mul_noLUT (x: u32, y: u32, prim: u32) -> u32 {
    let mut result = cl_mul(x, y);
    if prim > 0 {
        result = cl_div(result, prim);
    }
    return result;
}

pub fn bit_length (n: u32) -> u32 {
    return 32-n.leading_zeros();
}

pub fn cl_div (dividend: u32, divisor: u32) -> u32 {
    if dividend.leading_zeros() > divisor.leading_zeros() {
        return dividend;
    }
    
    let dl1 = 32-dividend.leading_zeros();
    let dl2 = 32-divisor.leading_zeros();
    let mut dividend = dividend;
    for i in (0..=(dl1-dl2)).rev() {
        if dividend & (1 << i+dl2-1) != 0 {
            dividend ^= divisor << i;
        }
    }
    return dividend;
}

// just the example taken from the tutorial
// returns 0001010001111010 and 0000000011000011 (correct)
pub fn test_gf () {
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
    println!("{:016b}", gf_mul_noLUT(a, b, 0));
    println!("{:016b}", gf_mul_noLUT(a, b, 0x11d));

    println!("{:016b}", gf_mul_noLUT_rpm(a, b, 0, 256, true));
    println!("{:016b}", gf_mul_noLUT_rpm(a, b, 0x11d, 256, true));
}

// uses russian peasant multiplication
// default values prim = 0 field_charac_full = 256, carryless = true
/*
    Galois Field integer multiplication using Russian Peasant Multiplication algorithm 
    (faster than the standard multiplication + modular reduction).
    If prim is 0 and carryless=False, then the function produces the result for 
    a standard integers multiplication (no carry-less arithmetics nor modular reduction).
*/
pub fn gf_mul_noLUT_rpm (x: u32, y: u32, prim: u32, field_charac_full: u32, carryless: bool) -> u32 {
    let mut x = x;
    let mut y = y;
    let mut r = 0;

    while y > 0 {
        if (y & 1) != 0 {
            r = if carryless {r^x} else {r+x};
        }
        y = y >> 1;
        x = x << 1;
        if prim > 0 && x & field_charac_full != 0 {
            x = x ^ prim;
        }
    }
    return r;
}