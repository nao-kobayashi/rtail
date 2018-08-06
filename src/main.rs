extern crate memmap;
extern crate encoding;

use std::env::args;
use std::fs::File;
use std::error::Error;
use memmap::MmapOptions;
use encoding::{ Encoding, DecoderTrap };
use encoding::all::WINDOWS_31J;

fn print_vec(buffer: Vec<u8>) {
    let lines: _ = buffer.split(|buf| *buf == b'\n')
        .filter(|buf| buf.len() > 0)
        .map(|buf| { encode(buf).unwrap_or("".to_owned()) });

    for line in lines {
        println!("{}", line);
    }
}

fn find_start_pos(mmap: &memmap::Mmap, length: usize, disp_rows: i32) -> usize {
    //ファイルサイズはsliceインデックスで見るから-1
    let mut index: usize = length - 1;
    let mut counter = disp_rows;
    while counter > 0 {
        if index <= 1 { return 0; }
        if &mmap[index..(index + 1)] == b"\n" {
            counter -= 1;
        }
        index -= 1;
    }
    //+1しないと先頭の改行が出てしまう。
    index + 1
}

fn read_file(file_path: &str, start_pos: Option<usize>, disp_rows: i32) -> Result<Option<(Vec<u8>, usize)>, Box<Error>> {
    let mmap = unsafe { 
        let file = File::open(file_path)?;
        MmapOptions::new().map(&file)? 
    };

    let length = *(&mmap.len() as &usize);
    let limit = if let Some(limit) = start_pos {
        limit
    } else {
        find_start_pos(&mmap, length, disp_rows)
    };

    if length <= limit { 
        Ok(None)
    } else {
        Ok(Some((mmap[limit..length].to_vec(), length)))
    }
}

fn encode(buffer: &[u8]) -> Option<String> {
    if let Some(utf) = encode_utf8(buffer) {
        Some(utf)
    } else {
        encode_shit_jis(buffer)
    }
}
    
fn encode_shit_jis(buffer: &[u8]) -> Option<String> {
    let mut chars = String::new();
    if let Ok(_) = WINDOWS_31J.decode_to(&buffer.to_vec(), DecoderTrap::Replace, &mut chars) {
        return Some(chars);
    }
    None
}

fn encode_utf8(buffer: &[u8]) -> Option<String> {
    if let Ok(output) = String::from_utf8(buffer.to_vec()) {
        return Some(output)
    }
    None
}

fn main() {
    let mut args = args().skip(1);
    //第1引数はファイル名（必須）
    let file_path = args.next().expect("ファイル名が指定されていません");
    //第2引数は初回の表示行数 デフォルト20行
    let disp_rows: i32 = if let Some(p) = args.next() { p.parse::<i32>().unwrap_or(20) } else { 20 };
    //第3引数はファイルの監視時間間隔 デフォルト3秒
    let refresh_sec: u64 = if let Some(p) = args.next() { p.parse::<u64>().unwrap_or(3) } else { 3 };

    let mut position = None;
    loop {
        //ファイルが書き換わったら表示する。
        match read_file(&file_path, position, disp_rows) {
            Ok(x) => {
                if let Some(x) = x  {
                    print_vec(x.0);
                    position = Some(x.1);
                } else {
                    std::thread::sleep(std::time::Duration::from_secs(refresh_sec));
                }
            },
            Err(e) => {
                println!("read_file error {:?}", e);
                std::process::exit(-1);
            }
        };
    }
}

