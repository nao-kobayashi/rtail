extern crate memmap;
extern crate encoding;

use std::fs::File;
use std::io::{ Write, stdout, Error };
use memmap::MmapOptions;
use encoding::{ Encoding, DecoderTrap };
use encoding::all::WINDOWS_31J;

struct Tail {
    file_path: String,
    display_rows: i32,
    current_pos: Option<usize>,
}

impl Tail {
    fn new(file_path: String, display_rows: i32) -> Self {
        Tail {
            file_path: file_path,
            display_rows: display_rows,
            current_pos: None,
        }
    }

    fn print_vec(&self, buffer: Vec<u8>) -> Result<(), Error> {
        let mut out = stdout();
        let enc_str = self.decode(buffer.as_ref()).unwrap_or("".to_owned());
        out.write_all(enc_str.as_bytes())?;
        out.flush()?;
        Ok(())
    }

    fn read_file(&mut self) -> Result<Option<Vec<u8>>, Box<Error>> {
        let file = File::open(self.file_path.clone())?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let length = *(&mmap.len() as &usize);

        let limit = if let Some(limit) = self.current_pos {
            if length <= limit {
                return Ok(None);
            }
            limit
        } else {
            self.find_start_pos(&mmap, length, self.display_rows)
        };
        
        self.current_pos = Some(length);
        Ok(Some(mmap[limit..length].to_vec()))
    }

    fn find_start_pos(&self, mmap: &memmap::Mmap, length: usize, mut counter: i32) -> usize {
        for i in (0..length).rev() {
            if mmap[i] == b'\n' {
                counter -= 1;
                if counter == 0 { return i + 1; }
            }
        }
        0
    }

    fn decode(&self, buffer: &[u8]) -> Option<String> {
        let mut output = String::new();
        if let Ok(output) = String::from_utf8(buffer.to_vec()) {
            return Some(output)
        } else if let Ok(_) = WINDOWS_31J.decode_to(&buffer.to_vec(), DecoderTrap::Replace, &mut output) {
            return Some(output);
        }
        None
    }
}

fn main() {
    //第1引数はファイル名（必須）
    //第2引数は初回の表示行数 デフォルト20行
    //第3引数はファイルの監視時間間隔 デフォルト3秒
    let mut args = std::env::args().skip(1);
    let file_path = args.next().expect("ファイル名が指定されていません");
    let mut tail = Tail::new(
        file_path, 
        if let Some(p) = args.next() { p.parse::<i32>().unwrap_or(20) } else { 20 }
    );
    let refresh_sec: u64 = if let Some(p) = args.next() { p.parse::<u64>().unwrap_or(3) } else { 3 };

    loop {
        let result = tail.read_file().expect("ファイルの読み込みに失敗しました");
        if let Some(result) = result {
            if let Err(e) = tail.print_vec(result) {
                panic!(format!("write error:{}", e));
            }
        } else {
            std::thread::sleep(std::time::Duration::from_secs(refresh_sec));
        }
    }
}

