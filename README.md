# *Q dot R es* – a QR generator in standard Rust ✨
*Q dot R es*, henceforth referred to by `qr`, is a QR code generator I wrote singlehandedly as a challenge for myself to improve at Rust. It's not intended to be an example of a fast, polished, or efficient QR generator. That being said, though, it does work reliably.

## Input
`qr` accepts input both as arguments and from `stdin`. The following commands should all result in the same code. 
* Manually choosing ASCII encoding:
```
qr --manual --ascii "Hello!"
```
* Letting the program choose the optimal encoding to minimize message size (which will result in ASCII encoding in this case):
```
qr -i "Hello!" 
```
* Piping text into the program (again choosing encoding automatically):
```
echo "Hello!" | qr
```
There is no CLI documentation at the moment. If you'd like to see what other arguments are available, please read through the `main_qr_generator` argument parsing code in [**`main.rs`**](src/main.rs).
### Unicode support
As of version 0.3, `qr` supports arbitrary Unicode characters as well. Any non-ASCII characters in the input (in either the manual ASCII mode or the automatic encoding mode) will add a UTF-8 marker to the QR code and divide UTF-8 characters into their constituent bytes.

As such, entering non-ASCII characters will increase the message size slightly, but the ASCII characters within the message will still be handled as normal.
```
qr --manual --ascii "Hello! 👋😌"
qr -i "I don't know… 😗🎶"
echo "😳💦 Are you sure?" | qr
```
## Output
`qr` outputs XBM bitmaps, which is an uncompressed monochrome format consisting of plaintext hex data wrapped in C code:
```c
#define hello_width 37
#define hello_height 37
static unsigned char hello_bits[] = {
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x7f, 0xdf, 0x1f, 0x00, 0x00, 0x41, 0x55,
    0x10, 0x00, 0x00, 0x5d, 0x41, 0x17, 0x00, 0x00, 0x5d, 0x4f, 0x17, 0x00,
    0x00, 0x5d, 0x50, 0x17, 0x00, 0x00, 0x41, 0x4b, 0x10, 0x00, 0x00, 0x7f,
    0xd5, 0x1f, 0x00, 0x00, 0x00, 0x1e, 0x00, 0x00, 0x00, 0x73, 0x8a, 0x1e,
    0x00, 0x00, 0xbf, 0xd6, 0x0f, 0x00, 0x00, 0x6a, 0x69, 0x09, 0x00, 0x00,
    0x97, 0xa5, 0x01, 0x00, 0x00, 0xf2, 0x32, 0x08, 0x00, 0x00, 0x00, 0x01,
    0x1b, 0x00, 0x00, 0x7f, 0xec, 0x0b, 0x00, 0x00, 0x41, 0xaf, 0x09, 0x00,
    0x00, 0x5d, 0x9f, 0x01, 0x00, 0x00, 0x5d, 0x64, 0x1b, 0x00, 0x00, 0x5d,
    0xc4, 0x03, 0x00, 0x00, 0x41, 0x2b, 0x00, 0x00, 0x00, 0x7f, 0xa9, 0x11,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00,
};
```
They are readable by free graphics software such as GIMP, though the ones output by this program are very small in size (less than 200 pixels wide) and will need to be rescaled.
## Code
Making a QR code involves:
* Reed-Solomon error correction math – [**`rdsm.rs`**](src/rdsm.rs)
   * Finite field arithmetic in GF(2⁸) – [**`galois.rs`**](src/rdsm/galois.rs)
   * Polynomial rings over GF(2⁸) – [**`poly.rs`**](src/rdsm/poly.rs)
   * Precomputed tables – [**`lookup.rs`**](src/rdsm/lookup.rs)
* Technical aspects of the QR standard – [**`qr_standard.rs`**](src/qr_standard.rs)
   * Reference tables – [**`tables.rs`**](src/qr_standard/tables.rs)
   * Binary bit stream handling – [**`badstream.rs`**](src/qr_standard/badstream.rs)
   * Higher-level character handling – [**`bitstream.rs`**](src/qr_standard/bitstream.rs)
      * A pathfinding algorithm for size optimization – [**`search.rs`**](src/qr_standard/bitstream/search.rs)
* Bitmap format handling – [**`image.rs`**](src/image.rs)

The documentation is far from finished, but some information can be gleaned using `cargo doc`.
## Notes (or: what `qr` is *not*)
1. During this project, I've deliberately tried to solve problems independently and not rely on others' solutions. As such, the `qr` code (🤭) may have some glaring faults due to me working off of incorrect information, or just not knowing any better. For the time being, I won't be seeking out others' code to compare against, but feel free to open an issue if you notice anything.

2. Beyond regular ASCII/byte data, QR codes have three special compressed encodings available: numeric, alphanumeric (a specific 45-character set), and Shift-JIS kanji. I've decided not to implement kanji support, for a few reasons:

   * It would demand a lot of extra work, as the rules for kanji encoding are very different from those of the other three sets,

   * I can't read Japanese, so I wouldn't use it and would also have trouble bugtesting it, and

   * UTF-8 support means that kanji characters will be encoded just fine regardless, albeit not *optimally.*

3. I haven't been able to check that the masking-pattern penalty routines work properly, as the QR standard isn't very clear about how the computation should be done, nor any reference values for specific patterns. I have done my best to interpret it.