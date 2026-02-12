pub mod cpu;

mod decoder {
    include!(concat!(env!("OUT_DIR"), "/gekko.rs"));
}

pub use decoder::GekkoInstruction;

use std::env;
use std::fs;
use std::process;

fn parse_offset(s: &str) -> Result<usize, std::num::ParseIntError> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        usize::from_str_radix(hex, 16)
    } else {
        s.parse()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: {} <ipl.bin> [offset]", args[0]);
        process::exit(1);
    }

    let data = fs::read(&args[1]).unwrap_or_else(|e| {
        eprintln!("failed to read {}: {}", args[1], e);
        process::exit(1);
    });

    let start = if args.len() >= 3 {
        parse_offset(&args[2]).unwrap_or_else(|e| {
            eprintln!("invalid offset '{}': {}", args[2], e);
            process::exit(1);
        })
    } else {
        0
    };

    if data.len() < start + 4 {
        eprintln!("file too small for offset {:#x}", start);
        process::exit(1);
    }

    let mut offset = start;
    while offset + 4 <= data.len() {
        let word = u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap());
        let addr = offset as u32;

        match GekkoInstruction::decode(word) {
            Some(instr) => println!("{:08X}  {:08X}  {}", addr, word, instr),
            None => println!("{:08X}  {:08X}  .long     {:#010x}", addr, word, word),
        }

        offset += 4;
    }
}
