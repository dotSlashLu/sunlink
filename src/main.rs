use std::fs::{self, OpenOptions};
use std::process::exit;
use std::thread::sleep;
use std::time;

const FILE: &str = "./t";
// const STEP: u64 = 1024 * 1024 * 300;
const STEP: u64 = 1000;
const SLEEP_SEC: u64 = 3;

fn main() {
    let meta = match fs::metadata(FILE) {
        Ok(meta) => meta,
        Err(e) => {
            println!("failed to get file metadata: {}", e);
            exit(1);
        }
    };

    let mut size = meta.len();
    let f_hd = match OpenOptions::new().write(true).open(FILE) {
        Ok(hd) => hd,
        Err(e) => {
            println!("failed to open file: {}", e);
            exit(1);
        }
    };

    while size > 0 {
        if size < STEP {
            must_set_len(&f_hd, 0);
            break;
        }
        must_set_len(&f_hd, size - STEP);
        size -= STEP;
        sleep(time::Duration::from_secs(SLEEP_SEC))
    }

    if let Err(e) = fs::remove_file(FILE) {
        println!("failed to remove file: {}", e);
    }
}

fn must_set_len(f_hd: &fs::File, len: u64) {
    match f_hd.set_len(len) {
        Err(e) => {
            println!("failed to truncate: {}", e);
            exit(1);
        }
        _ => (),
    };
}
