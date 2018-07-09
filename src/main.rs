extern crate memmap;
extern crate encoding;

use std::env::args;
use std::fs::File;
use memmap::MmapOptions;
use encoding::{ Encoding, DecoderTrap };
use encoding::all::WINDOWS_31J;

fn get_read_start_pos(mmap: &memmap::Mmap, length: usize, disp_rows: i32) -> usize {
    let mut index: usize = length - 1;
    let mut counter = disp_rows;

    loop {
        if index <= 1 { 
            return 0;
        }

        let first_byte = index - 1;
        let second_byte = index;
        if &mmap[first_byte..first_byte + 1] == b"\r" && &mmap[second_byte..second_byte + 1] == b"\n" {
            index = index - 1;
            counter = counter - 1;
            if counter < 0 { 
                //最後に見つかった改行分は出力不要
                index = index + 2;
                break; 
            }
        } else {
            index = index - 1;
        }
    }

    index
}

fn print_vec(buffer: Vec<u8>) {
    let mut index = 0;
    let mut output_vec: Vec<u8> = Vec::new();
    let slice_len = buffer.len();

    while index < slice_len  {
        if (index + 1) < slice_len && &buffer[index] == &b"\r"[0] && &buffer[index + 1] == &b"\n"[0] {
            let copy_output_vec_utf = output_vec.clone();
            let copy_output_vec_iso = output_vec.clone();
            let cnv_string = if let Ok(output) = String::from_utf8(copy_output_vec_utf) {
                output
            } else {
                let mut chars = String::new();
                match WINDOWS_31J.decode_to(&copy_output_vec_iso, DecoderTrap::Replace, &mut chars) {
                    Ok(_) => {},
                    Err(e) => {
                        println!("parse error {:?}", e);
                        std::process::exit(-1);
                    }
                };

                chars
            };

            println!("{}", cnv_string);
            output_vec.clear();
            //最後に見つかった改行分は出力不要
            index = index + 2;
        } else {
            output_vec.push(buffer[index]);
            index = index + 1;
        }
    }
}

fn read_file(file_path: &str, disp_rows: i32) -> ReadResult {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            panic!(format!("{:?}", e));
        }
    };

    let mmap = unsafe { 
        match MmapOptions::new().map(&file) {
            Ok(map) => map,
            Err(e) => {
                panic!(format!("{:?}", e));
            }
        }
    };

    let length = *(&mmap.len() as &usize);
    let start_pos = get_read_start_pos(&mmap, length, disp_rows);

    ReadResult {
        read_buffer: mmap[start_pos..length].to_vec(),
        buf_length: length,
    }
}

fn read_file_remain_all(file_path: &str, start_pos: usize) -> Option<ReadResult> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            panic!(format!("{:?}", e));
        }
    };

    let mmap = unsafe { 
        match MmapOptions::new().map(&file) {
            Ok(map) => map,
            Err(e) => {
                panic!(format!("{:?}", e));
            }
        }
    };

    let length = *(&mmap.len() as &usize);

    if length <= start_pos {
        return None;
    }

    Some(ReadResult {
        read_buffer: mmap[start_pos..length].to_vec(),
        buf_length: length,
    })
}

struct ReadResult {
    read_buffer: Vec<u8>,
    buf_length: usize,
}

fn main() {
    let args: Vec<String> = args().collect();
    let file_path = &args[1];
    let disp_rows: i32 = args[2].parse().unwrap();

    let read_result = read_file(file_path, disp_rows);
    let mut length = read_result.buf_length;
    print_vec(read_result.read_buffer);

    loop {
        //ファイルが書き換わったら表示する。
        match read_file_remain_all(file_path, length) {
            Some(x) => {
                print_vec(x.read_buffer);
                length = x.buf_length;
            },
            None => {
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
        };
    }
}
