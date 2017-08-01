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

use std::path;
use std::fs;
use std::io::prelude::*;


fn load_input_file(in_path: &path::Path, buffer: &mut Vec<u8>) -> Option<usize> {
    let mut in_file = match fs::File::open(in_path) {
        Ok(file) => { file },
        Err(error) => {
            eprintln!("Failed to open `{}' for reading: {}.",
                      in_path.display(), error);
            return None;
        },
    };
    match in_file.read_to_end(buffer) {
        Ok(length) => { Some(length) },
        Err(error) => {
            eprintln!("Failed to load the content of `{}': {}.",
                      in_path.display(), error);
            None
        },
    }
}

fn write_down_tape_file(output_buffer: &Vec<u8>, out_path: &path::Path) -> bool {
    let mut out_file = match fs::File::create(out_path) {
        Ok(file) => { file },
        Err(error) => {
            eprintln!("Failed to open `{}' for writing: {}.",
                      out_path.display(), error);
            return false;
        },
    };
    match out_file.write_all(output_buffer.as_slice()) {
        Ok(()) => {
            println!("");
            println!("Successfully wrote {} bytes into `{}'.",
                     output_buffer.len(), out_path.display());
            true
        },
        Err(error) => {
            eprintln!("Failed to save the created tape into `{}': {}.",
                      out_path.display(), error);
            false
        },
    }
}

fn input_file_sanity_check(in_path: &path::Path, base_address: u16, length: usize) -> bool {
    println!("{}: {} bytes loaded.", in_path.display(), length);

    if length > (0x10000 - (base_address as usize)) {
        println!("");
        eprintln!("The input file would not fit into the Z80's address space.");
        eprintln!("With a base address of 0x{:04X}, you can only fit at most {} bytes.", base_address, (0x10000 - (base_address as usize)));

        false
    } else if length == 0 {
        println!("");
        eprintln!("The input file is empty, there's nothing to write onto the tape.");

        false
    } else {
        true
    }
}

fn generate_data_entry_header(entry_name: &[u8], buffer: &mut Vec<u8>) {
    buffer.reserve(256 + 2 + 6);

    // Tape Leader:
    for _counter in 0..256 {
        buffer.push(0);
    }

    // Sync byte:
    buffer.push(0xA5);

    // Header byte indicating system format:
    buffer.push(0x55);

    // 6 character file name in ASCII:
    for count in 0..6 {
        buffer.push(entry_name[count]);
    }
}

fn pack_chunk(chunk_to_pack: &[u8], output_buffer: &mut Vec<u8>, load_address: u16) -> usize {
    output_buffer.reserve(5 + chunk_to_pack.len());
    let mut checksum: u8 = 0;

    // Data header:
    output_buffer.push(0x3C);

    // Length of data, 0 = 256:
    match chunk_to_pack.len() {
        256 => { output_buffer.push(0); },
        _ => { output_buffer.push(chunk_to_pack.len() as u8); },
    }

    // lsb, msb of the load address:
    output_buffer.push((load_address & 0x00FF) as u8);
    output_buffer.push(((load_address & 0xFF00) >> 8) as u8);

    checksum = checksum.wrapping_add((load_address & 0x00FF) as u8);
    checksum = checksum.wrapping_add(((load_address & 0xFF00) >> 8) as u8);

    for chunk_iter in 0..chunk_to_pack.len() {
        output_buffer.push(chunk_to_pack[chunk_iter]);
        checksum = checksum.wrapping_add(chunk_to_pack[chunk_iter]);
    }

    // A checksum of the data and the load address:
    output_buffer.push(checksum);

    // Return the size of the packed chunk:
    chunk_to_pack.len()
}

fn pack_binary_image(input_buffer: &Vec<u8>, output_buffer: &mut Vec<u8>, base_address: u16) {
    let binary_image_length = input_buffer.len();
    let mut already_packed: usize = 0;
    let mut full_chunks_count: usize = 0;
    let mut last_chunk_size: Option<usize> = None;

    while already_packed < binary_image_length {
        if (binary_image_length - already_packed) > 255 {
            already_packed += pack_chunk(&input_buffer[already_packed..already_packed+256], output_buffer, base_address + (already_packed as u16));
            full_chunks_count += 1;
        } else {
            last_chunk_size = Some(pack_chunk(&input_buffer[already_packed..], output_buffer, base_address + (already_packed as u16)));
            already_packed += last_chunk_size.unwrap();
        }
    }

    match last_chunk_size {
        Some(size) => {
            println!("Packed {} chunks of 256 bytes and 1 chunk of {} bytes.",
                     full_chunks_count, size);
        },
        None => {
            println!("Packed {} chunks of 256 bytes.", full_chunks_count);
        },
    }
}

fn finalize_data_entry(entry_point: u16, output_buffer: &mut Vec<u8>) {
    // End of file marker:
    output_buffer.push(0x78);

    // lsb, msb of the entry point:
    output_buffer.push((entry_point & 0x00FF) as u8);
    output_buffer.push(((entry_point & 0xFF00) >> 8) as u8);
}


pub fn pack(in_path: &path::Path, out_path: &path::Path, entry_name: &[u8],
            base_address: u16, entry_point: u16) -> bool {
    assert!(entry_name.len() == 6);

    let mut input_buffer  = Vec::new();
    let mut output_buffer = Vec::new();

    match load_input_file(in_path, &mut input_buffer) {
        Some(length) => {
            assert!(length == input_buffer.len());
            if !input_file_sanity_check(in_path, base_address, length) {
                return false;
            }
        }
        None => {
            return false;
        }
    }
    generate_data_entry_header(entry_name, &mut output_buffer);
    pack_binary_image(&input_buffer, &mut output_buffer, base_address);
    finalize_data_entry(entry_point, &mut output_buffer);

    write_down_tape_file(&output_buffer, out_path)
}
