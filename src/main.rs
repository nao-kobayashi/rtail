extern crate memmap;
extern crate encoding;

use std::fs::File;
use std::error::Error;
use memmap::MmapOptions;
use encoding::{ Encoding, DecoderTrap };
use encoding::all::WINDOWS_31J;

fn print_vec(buffer: Vec<u8>) {
    buffer.split(|buf| *buf == b'\n')
        .filter(|buf| buf.len() > 0)
        .map(|buf| decode(buf).unwrap_or("".to_owned()))
        .for_each(|line| println!("{}", line));
}

fn find_start_pos(mmap: &memmap::Mmap, length: usize, mut counter: i32) -> usize {
    for i in (1..length).rev() {
        if mmap[i] == b'\n' {
            counter -= 1;
            if counter == 0 { return i + 1; }
        }
    }
    0
}

fn read_file(file_path: &str, start_pos: Option<usize>, disp_rows: i32) -> Result<Option<(Vec<u8>, usize)>, Box<Error>> {
    let file = File::open(file_path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let length = *(&mmap.len() as &usize);

    let limit = if let Some(limit) = start_pos {
        limit
    } else {
        find_start_pos(&mmap, length, disp_rows)
    };

    if length <= limit {
        return Ok(None);
    }
    Ok(Some((mmap[limit..length].to_vec(), length)))
}

fn decode(buffer: &[u8]) -> Option<String> {
    if let Some(utf) = decode_utf8(buffer) {
        Some(utf)
    } else {
        decode_shift_jis(buffer)
    }
}

fn decode_shift_jis(buffer: &[u8]) -> Option<String> {
    let mut chars = String::new();
    if let Ok(_) = WINDOWS_31J.decode_to(&buffer.to_vec(), DecoderTrap::Replace, &mut chars) {
        return Some(chars);
    }
    None
}

fn decode_utf8(buffer: &[u8]) -> Option<String> {
    if let Ok(output) = String::from_utf8(buffer.to_vec()) {
        return Some(output)
    }
    None
}

fn main() {
    let mut args = std::env::args().skip(1);
    let file_path = args.next().expect("ファイル名が指定されていません");    //第1引数はファイル名（必須）
    let disp_rows: i32 = if let Some(p) = args.next() { p.parse::<i32>().unwrap_or(20) } else { 20 };   //第2引数は初回の表示行数 デフォルト20行
    let refresh_sec: u64 = if let Some(p) = args.next() { p.parse::<u64>().unwrap_or(3) } else { 3 };   //第3引数はファイルの監視時間間隔 デフォルト3秒

    let mut position = None;
    loop {
        let result = read_file(&file_path, position, disp_rows).expect("ファイルの読み込みに失敗しました");
        if let Some(result) = result {
            print_vec(result.0);
            position = Some(result.1);
        } else {
            std::thread::sleep(std::time::Duration::from_secs(refresh_sec));
        }
    }
}

