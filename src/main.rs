use std::fs::{self, OpenOptions};
use std::process::exit;
use std::thread::sleep;
use std::time;

use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};

const DEFAULT_STEP: &str = "1000";
const DEFAULT_SLEEP_SEC: &str = "1.0";

fn main() {
    let matches = App::new("SUNLINK")
        .version("1.0")
        .author("Pan1c <qiang@pan1c.org>")
        .about("Safely delete large files")
        .arg(
            Arg::with_name("sleep")
                .short("d")
                .long("sleep")
                .value_name("DURATION")
                .help("Sleep duration in seconds between each chunk truncation")
                .default_value(DEFAULT_SLEEP_SEC)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("chunk_size")
                .short("c")
                .long("chunk-size")
                .value_name("SIZE")
                .help("max size in MB for each truncation")
                .default_value(DEFAULT_STEP)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("FILE")
                .help("file to delete")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("silent")
                .short("s")
                .long("silent")
                .multiple(true)
                .help("no progress bar"),
        )
        .get_matches();

    let f = matches.value_of("FILE").unwrap();
    let silent = matches.occurrences_of("silent") == 1;
    let sleep_duration = match matches.value_of("sleep") {
        Some(d) => match d.parse::<f32>() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("filed to parse `sleep': {}", e);
                exit(1);
            }
        },
        None => DEFAULT_SLEEP_SEC.parse().unwrap()
    };
    let step = match matches.value_of("chunk_size") {
        Some(s) => match s.parse::<u64>() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("failed to parse `chunk_size': {}", e);
                exit(1);
            }
        },
        None => DEFAULT_STEP.parse::<u64>().unwrap(),
    };
    // Megabyte => byte
    // let step = step * 1000 * 1000;
    let step = step;

    let (mut size, f_hd) = match finfo(f) {
        Ok(info) => info,
        Err(e) => {
            eprintln!("failed to get file info: {}", e);
            exit(1);
        }
    };

    let bar = if silent {
        None
    } else {
        let bar = ProgressBar::new(size);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar} {pos:>7}/{len:7} {msg}"),
        );
        Some(bar)
    };

    while size > 0 {
        if size < step {
            must_set_len(&f_hd, 0);
            if bar.is_some() {
                bar.as_ref().unwrap().inc(size);
            }
            break;
        }
        must_set_len(&f_hd, size - step);
        size -= step;
        if bar.is_some() {
            bar.as_ref().unwrap().inc(step);
        }
        sleep(time::Duration::from_secs_f32(sleep_duration))
    }

    if let Err(e) = fs::remove_file(f) {
        eprintln!("failed to remove file: {}", e);
    }
}

fn finfo(f: &str) -> Result<(u64, Box<fs::File>), std::io::Error> {
    let meta = fs::metadata(f)?;
    let size = meta.len();
    let f_hd = Box::new(OpenOptions::new().write(true).open(f)?);
    Ok((size, f_hd))
}

fn must_set_len(f_hd: &fs::File, len: u64) {
    if let Err(e) = f_hd.set_len(len) {
        println!("failed to truncate file: {}", e);
        exit(1);
    };
}
