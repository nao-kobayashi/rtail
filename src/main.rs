extern crate memmap;
extern crate encoding;

use std::fs::File;
use std::io::{ Write, stdout, Error };
use memmap::MmapOptions;
use encoding::{ Encoding, DecoderTrap };
use encoding::all::WINDOWS_31J;

fn print_vec(buffer: Vec<u8>) -> Result<(), Error> {
    let mut out = stdout();
    let enc_str = decode(buffer.as_ref()).unwrap_or("".to_owned());
    out.write_all(enc_str.as_bytes())?;
    out.flush()?;
    Ok(())
}

fn find_start_pos(mmap: &memmap::Mmap, length: usize, mut counter: i32) -> usize {
    for i in (0..length).skip(1).rev() {
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
    let mut output = String::new();

    if let Ok(output) = String::from_utf8(buffer.to_vec()) {
        return Some(output)
    } else if let Ok(_) = WINDOWS_31J.decode_to(&buffer.to_vec(), DecoderTrap::Replace, &mut output) {
        return Some(output);
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
            if let Err(e) = print_vec(result.0) {
                panic!(format!("write error:{}", e));
            }
            position = Some(result.1);
        } else {
            std::thread::sleep(std::time::Duration::from_secs(refresh_sec));
        }
    }
}

