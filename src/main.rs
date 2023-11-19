//! The *"Q-dot-R-es"* QR code generator.

mod image;
mod qr_standard;
mod rdsm;

use qr_standard::Mode::*;

use image::Bitmap;
use qr_standard::*;
use rdsm::*;

fn main() -> std::io::Result<()> {
    main_qr_generator()
}

fn main_qr_generator() -> std::io::Result<()> {
    let mut input_choice: Option<QRInput> = None;
    let mut level_choice = Option::<u8>::None;
    let mut mask_choice = Option::<u8>::None;
    let mut name = Option::<String>::None;

    let mut manual: bool = true;
    let mut mode_data = Vec::new();
    let mut version_choice = Option::<u32>::None;

    let mut args = std::env::args();

    args.next();
    while let Some(argument) = args.next() {
        if manual && ["--manual"].contains(&argument.as_str()) {
            'goop: while let Some(argument) = args.next() {
                match argument.as_str() {
                    "--numeric" | "-num" | "" => {
                        let number_string = args.next().expect("no data for numeric mode");
                        mode_data.push((Numeric, number_string));
                    }
                    "--alphanum" | "-aln" => {
                        let alphanum_string = args.next().expect("no data for alphanumeric mode");
                        mode_data.push((AlphaNum, alphanum_string));
                    }
                    "--ascii" | "-asc" => {
                        let ascii_string = args.next().expect("no data for ASCII mode");
                        mode_data.push((ASCII, ascii_string));
                    }
                    _ => {
                        if argument.starts_with('-') {
                            assert!(!mode_data.is_empty(), "manual input not specified");
                            manual = false;
                            break 'goop;
                        } else {
                            continue;
                        }
                    }
                }
            }
        }

        match argument.as_str() {
            "--input" | "-i" => {
                assert!(
                    mode_data.is_empty() && manual,
                    "can't combine manual and automatic mode switching!"
                );
                assert!(input_choice.is_none(), "can't use multiple inputs!");

                manual = false;
                let auto_string = args.next().expect("no data for ASCII mode");
                input_choice = Some(QRInput::Auto(auto_string));
            }

            "--name" | "-n" => {
                if name.is_none() {
                    name = Some(args.next().expect("no name submitted"));
                } else {
                    panic!("can't specify name twice")
                }
            }
            "--level" | "-l" => {
                if level_choice.is_none() {
                    level_choice = Some(
                        match args
                            .next()
                            .expect("no error correction level submitted")
                            .to_ascii_lowercase()
                            .trim()
                        {
                            "l" => 0,
                            "m" => 1,
                            "q" => 2,
                            "h" => 3,
                            _ => panic!("invalid error correction level"),
                        },
                    );
                } else {
                    panic!("can't specify level twice")
                }
            }
            "--version" | "-v" => {
                if version_choice.is_none() {
                    version_choice = Some(
                        args.next()
                            .expect("no version submitted")
                            .parse::<u32>()
                            .expect("invalid version"),
                    );
                } else {
                    panic!("can't specify version twice")
                }
            }
            "--mask" | "-m" => {
                if mask_choice.is_none() {
                    mask_choice = Some(
                        args.next()
                            .expect("no mask submitted")
                            .parse::<u8>()
                            .expect("invalid mask"),
                    );
                } else {
                    panic!("can't specify mask twice")
                }
            }

            "--manual" => {}
            _ => panic!("{} - incorrect argument", argument),
        }
    }

    if let Some(v) = version_choice {
        assert!((1..=40).contains(&v), "version must be one of 1, ..., 40");
    }
    if let Some(m) = mask_choice {
        assert!((0..=7).contains(&m), "mask must be one of 0, ..., 7");
    }

    let input = if let Some(i) = input_choice {
        i
    } else {
        if mode_data.is_empty() {
            // if no manual mode data has been read from the arguments,
            // we get unprocessed data from stdin instead
            let mut input_string = String::new();
            std::io::stdin().read_line(&mut input_string)?;
            // stdin input ends on a newline, remove it
            input_string.pop();
            QRInput::Auto(input_string)
        } else {
            QRInput::Manual(mode_data)
        }
    };

    let name = name.unwrap_or("out".to_string());
    let output = make_qr(input, version_choice, level_choice, mask_choice).as_xbm(&name, true);
    std::fs::write(format!("{}.xbm", name), output)
}

#[test]
fn test_gauntlet() {
    for i in 0..100 {
        print!("{}...", i);
        {
            const NUM_LIST: &[usize] = &[
            // 20, 21, 23, 42, 43, 44, 45, 71, 72, 73, 76, 104, 105, 107, 138, 139, 140, 141, 172,
        ];
            /* for i in *(NUM_LIST.last()).unwrap_or(&0)..1000 */
            {
                // print!("{} ... ", i);
                // if NUM_LIST.contains(&i) {
                //     println!("skipped");
                //     continue;
                // }
                let a = std::iter::repeat('1').take(i).collect::<String>();
                make_qr(QRInput::Auto(a), None, None, None);
                // println!("good");
            }
        }
        {
            const ALN_LIST: &[usize] = &[
            // 13, 14, 26, 27, 43, 44, 64, 65, 84, 85, 87, 104, 105, 106, 122, 123, 154, 155, 186,
        ];
            /* for i in *(ALN_LIST.last()).unwrap_or(&0)..1000 */
            {
                // print!("{} ... ", i);
                // if ALN_LIST.contains(&i) {
                //     println!("skipped");
                //     continue;
                // }
                let a = std::iter::repeat('A').take(i).collect::<String>();
                make_qr(QRInput::Auto(a), None, None, None);
                // println!("good");
            }
        }
        {
            const ASC_LIST: &[usize] = &[
            // 9, 10, 11, 18, 19, 20, 30, 31, 32, 44, 45, 46, 58
            ];
            /* for i in *(ASC_LIST.last()).unwrap_or(&0)..1000 */
            {
                // print!("{} ... ", i);
                // if ASC_LIST.contains(&i) {
                //     println!("skipped");
                //     continue;
                // }
                let a = std::iter::repeat('a').take(i).collect::<String>();
                make_qr(QRInput::Auto(a), None, None, None);
                // println!("good");
            }
        }
        {
            const MAN_LIST: &[usize] = &[/* 5 */];
            /* for i in *(MAN_LIST.last()).unwrap_or(&0)..1000 */
            {
                // print!("{} ... ", i);
                // if MAN_LIST.contains(&i) {
                //     println!("skipped");
                //     continue;
                // }
                let a = [(ASCII, format!("1")), (AlphaNum, format!("1"))]
                    .into_iter()
                    .cycle()
                    .take(i)
                    .collect::<Vec<_>>();
                // let a = std::iter::repeat('a').take(i).collect::<String>();
                make_qr(QRInput::Manual(a), None, None, None);
                // println!("good");
            }
        }
        println!(" good");
    }
}
