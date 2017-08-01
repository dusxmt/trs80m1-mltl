// Copyright (c) 2017 Marek Benc <dusxmt@gmx.com>
//
// Permission to use, copy, modify, and distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
//

extern crate getopts;

mod packing;

use std::env;
use std::path;
use std::process;


fn print_usage(progname: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} [options] -i <file> -b <base_addr> -s <entry_point>", progname);
    println!("{}", opts.usage(&brief));
}

// Figure out the name of the executable:
fn get_progname(arg0: &path::Path) -> String {

    match arg0.file_name() {
        Some(name_osstr) => {
            name_osstr.to_string_lossy().into_owned()
        },

        // If we can't figure it out, just guess.
        None => {
            "trs80m1-mltl".to_owned()
        },
    }
}


// Parse an unsigned hexadecimal number (command-line argument), with or without
// an optional 0x prefix:
fn parse_hex_arg(arg: &str) -> Option<u32> {
    let mut chars = arg.chars();

    let first_char = chars.next();
    match first_char {
        Some(first_char_ch) => {
            let effective_first_char: char;

            // Handle an optional '0x' prefix:
            if first_char_ch == '0' {
                effective_first_char = match chars.next() {
                    Some(character) => {
                        if character == 'x' || character == 'X' {
                            '0'
                        } else {
                            character
                        }
                    }
                    None => { '0' },
                }
            } else {
                effective_first_char = first_char_ch;
            }

            let mut accumulator: u32 = match effective_first_char.to_digit(16) {
                Some(digit) => { digit },
                None => { return None },
            };

            for current_char in chars {
                accumulator = match current_char.to_digit(16) {
                    Some(digit) => { (accumulator << 4) | digit },
                    None => { return None },
                };
            }

            Some(accumulator)
        },
        None => { None },
    }
}
fn retrieve_base_address(progname: &str, matches: &getopts::Matches) -> Option<(bool, u16)> {
    match matches.opt_str("b") {
        Some(argument) => {
            match parse_hex_arg(&argument) {
                Some(address) => {
                    if address > 0xFFFF {
                        eprintln!("{}: The specified base address 0x{:04X} doesn't fit into the Z80's address space.", progname, address);
                        None
                    } else {
                        Some((true, address as u16))
                    }
                }
                None => {
                    eprintln!("{}: Failed to parse the base address argument `{}'.", progname, argument);
                    None
                },
            }
        },
        None => {
            eprintln!("{}: Base address not speciified, please provide it with the `--base' command-line option.", progname);
            Some((false, 0))
        },
    }
}
fn retrieve_entry_point(progname: &str, matches: &getopts::Matches) -> Option<(bool, u16)> {
    match matches.opt_str("s") {
        Some(argument) => {
            match parse_hex_arg(&argument) {
                Some(address) => {
                    if address > 0xFFFF {
                        eprintln!("{}: The specified entry point address 0x{:04X} doesn't fit into the Z80's address space.", progname, address);
                        None
                    } else {
                        Some((true, address as u16))
                    }
                }
                None => {
                    eprintln!("{}: Failed to parse the entry point address argument `{}'.", progname, argument);
                    None
                },
            }
        },
        None => {
            eprintln!("{}: Entry point not speciified, please provide it with the `--start' command-line option.", progname);
            Some((false, 0))
        },
    }
}
// The return value is (name, contains_letters)
fn retrieve_tape_entry_name(default: &str, matches: &getopts::Matches) -> (Vec<u8>, bool) {

    let template = match matches.opt_str("n") {
        Some(argument) => { argument.to_owned() },
        None => { default.to_owned() },
    };

    let mut entry_name = vec![0x20; 6];
    let mut name_iter: usize = 0;
    let mut has_first_char = false;

    for character in template.chars() {
        if name_iter == 6 {
            break;
        }
        // Rust strings are Unicode, but here, we need ASCII, and only
        // letters and the space.
        //
        // Thankfully, ASCII is a subset of Unicode, and we can simply ignore
        // anything which doesn't fit our criteria.
        //
        let char_val = character as u32;

        let (new_byte, add_char) = if (char_val == 0x20) && has_first_char {
            (0x20, true)
        } else if (char_val >= 0x41) && (char_val <= 0x5A) {
            has_first_char = true;
            (char_val as u8, true)
        } else if (char_val >= 0x61) && (char_val <= 0x7A) {
            has_first_char = true;
            ((char_val - 0x20) as u8, true)
        } else {
            (0, false)
        };

        if add_char {
            entry_name[name_iter] = new_byte;
            name_iter += 1;
        }
    }

    assert!(entry_name.len() == 6);
    (entry_name, has_first_char)
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let progname = get_progname(args[0].as_ref());

    let mut options = getopts::Options::new();

    options.optopt("i", "input", "The file to pack into a machine language tape file.", "FILE");
    options.optopt("o", "output", "Name of the destination file (input filename with extension changed to .cas by default).", "FILE");
    options.optopt("b", "base", "Starting address of where the data will reside after being loaded (in hex).", "ADDR");
    options.optopt("s", "start", "Address of the execution entry point (in hex).", "ADDR");
    options.optopt("n", "name", "Name of the data entry on the tape (input filename without extension by default). The name is limited to 6 upper-case ASCII letters. The first 6 ASCII letters are converted to upper-case, and everything else except for spaces is stripped.", "NAME");
    options.optflag("h", "help", "Show this help listing.");

    let matches = match options.parse(&args[1..]) {
        Ok(matches) => { matches },
        Err(error) => {
            eprintln!("{}: Argument parsing error: {}", progname, error);
            process::exit(1);
        },
    };

    // Help should always be handled first:
    if matches.opt_present("h") {
        print_usage(&progname, options);
        process::exit(0);
    }

    // Mandatory arguments:
    let mut missing_mand_arg = false;
    let in_filepath = match matches.opt_str("i") {
        Some(name) => {
            let new_path = (name.as_ref() as &path::Path).to_owned();
            if !new_path.is_file() {
                eprintln!("{}: The specified input file `{}' is not a file.", progname, new_path.display());
                process::exit(1);
            }

            new_path
        },
        None => {
            eprintln!("{}: Input file not specified, please provide it with the `--input' command-line option.", progname);
            missing_mand_arg = true;
            ("".as_ref() as &path::Path).to_owned()
        }
    };

    let base_address = match retrieve_base_address(&progname, &matches) {
        Some((found, address)) => {
            if !found {
                missing_mand_arg = true;
            }
            address
        },
        None => { process::exit(1); },
    };
    let entry_point = match retrieve_entry_point(&progname, &matches) {
        Some((found, address)) => {
            if !found {
                missing_mand_arg = true;
            }
            address
        },
        None => { process::exit(1); },
    };

    if missing_mand_arg {
        eprintln!("");
        eprintln!("Some mandatory command-line options are missing, see `{} --help'.", progname);

        process::exit(1);
    }

    // Optional arguments:

    // The filename of the input filepath is used for defaults of optional
    // arguments.
    //
    // I feel that unwrap is reasonable here because we've already checked
    // that this is indeed a file, and that the argument is present.
    //
    let in_filename = (in_filepath.file_name().unwrap().as_ref() as &path::Path).to_owned();

    let out_filepath = match matches.opt_str("o") {
        Some(name) => { (name.as_ref() as &path::Path).to_owned() },
        None => {
            let mut new_name = in_filename.clone();
            new_name.set_extension("cas");

            new_name
        }
    };
    let mut default_entry_name = in_filename.clone();
    default_entry_name.set_extension("");
    let (tape_entry_name, name_has_letters) = retrieve_tape_entry_name(&default_entry_name.to_string_lossy().into_owned(), &matches);


    println!("Input filename:       `{}'", in_filepath.display());
    println!("Output filename:      `{}'", out_filepath.display());
    println!("Tape data entry name: `{}'", String::from_utf8((&tape_entry_name).to_owned()).expect("invalid characters in the tape data entry name, these should've been filtered out"));
    println!("Base address:          0x{:04X}", base_address);
    println!("Entry point address:   0x{:04X}", entry_point);
    println!("");

    if in_filepath == out_filepath {
        eprintln!("The input and output files are the same, aborting to prevent data loss.");
        process::exit(1);
    }
    if !name_has_letters {
        eprintln!("The name of the data entry to be \"recorded onto the tape\" is empty, this could be because there either are no plain ASCII letters in your input filename, or in the name you provided via the `--name' command-line option.");
        eprintln!("");
        eprintln!("Please provide a valid name for the data entry, see `{} --help'.", progname);

        process::exit(1);
    }

    // Perform the packing:
    if packing::pack(&in_filepath, &out_filepath, tape_entry_name.as_slice(),
                     base_address, entry_point) {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
