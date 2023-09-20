mod export;
mod bitmask;
mod zigzag;
mod image_type;

// program flow:
// generate blank matrix
// write a message to it
//   (what format is the message??)
// along with error correction bits
//   (are those included in the message,
//    or separate?)
// write format information to it
//   (at what step, what format?)
// apply bitmask
//   (which mask????)
// output

// re: format of the message, i want it to be
// a list of bytes, like 0f a6 42 etc.


fn main() {
    let a: String = format!("{:b}",13);
    
    for n in a.chars() {
        println!("{n}");
    }
}
