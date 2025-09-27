use qr::{interface, qr_standard};

fn main() -> std::io::Result<()> {
    {
        use qr_standard::{badstream::QRInput, bitstream::Mode};

        let mut input_choice: Option<QRInput> = None;
        let mut level_choice = Option::<u8>::None;
        let mut mask_choice = Option::<u8>::None;
        let mut name = Option::<String>::None;

        let mut manual: bool = true;
        let mut mode_data = Vec::new();
        let mut version_choice = Option::<u32>::None;

        let mut args = std::env::args();
        // let args_list = std::env::args().collect::<Vec<String>>();

        let mut first_loop = true;
        args.next();
        while let Some(argument) = args.next() {
            // hack to allow for printing help text without suppressing it elsewhere
            if first_loop && ["-h", "--help"].contains(&argument.as_str()) {
                println!("{}", interface::HELPTEXT);
                return Ok(());
            } else {
                first_loop = false
            }

            if manual && ["--manual"].contains(&argument.as_str()) {
                'goop: while let Some(argument) = args.next() {
                    match argument.as_str() {
                        "--numeric" | "-num" | "" => {
                            let number_string = args.next().expect("no data for numeric mode");
                            mode_data.push((Mode::Numeric, number_string));
                        }
                        "--alphanum" | "-aln" => {
                            let alphanum_string =
                                args.next().expect("no data for alphanumeric mode");
                            mode_data.push((Mode::AlphaNum, alphanum_string));
                        }
                        "--ascii" | "-asc" => {
                            let ascii_string = args.next().expect("no data for ASCII mode");
                            mode_data.push((Mode::ASCII, ascii_string));
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
                "--help" | "-h" => panic!("{} can't be used after other arguments", argument),
                _ => panic!("{} - incorrect argument", argument),
            }
        }

        if let Some(v) = version_choice {
            assert!((1..=40).contains(&v), "version must be one of 1, ..., 40");
        }
        if let Some(m) = mask_choice {
            assert!((0..=7).contains(&m), "mask must be one of 0, ..., 7");
        }

        let input = match input_choice {
            Some(i) => i,
            None => {
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
            }
        };

        let name = name.unwrap_or("out".to_string());
        let output =
            qr_standard::badstream::make_qr(input, version_choice, level_choice, mask_choice)
                .as_xbm(&name, true);

        let write_status = std::fs::write(format!("{}.xbm", name), output);
        if write_status.is_ok() {
            println!("Wrote '{name}.xbm' successfully.")
        }
        write_status
    }
}

// returns a description of inputs that will lead make_qr() to panic
#[test]
fn depanic() -> Result<(), String> {
    use qr_standard::badstream::QRInput;

    let check = |x: QRInput| {
        std::panic::catch_unwind(|| qr_standard::badstream::make_qr(x, None, None, Some(0)))
    };
    let make_string = |str: &str, i: usize| str.chars().cycle().take(i).collect::<String>();

    let mut offenders: Vec<(String, usize)> = vec![];
    if check(QRInput::Auto("".to_string())).is_err() {
        offenders.push(("empty string".to_string(), 0));
    }
    for i in 1..50 {
        for str in [
            "a", "A", "1", "A1", "a1", "aA", // normal
            "🤔", "π", // wild
        ] {
            let a = make_string(str, i);
            if check(QRInput::Auto(a)).is_err() {
                offenders.push((str.to_string(), i));
            }
        }
    }

    if offenders.is_empty() {
        Ok(())
    } else {
        let str = offenders
            .iter()
            .map(|(str, i)| format!("('{}', {}) ", str, i))
            .collect::<String>();
        Err(str)
    }
}
