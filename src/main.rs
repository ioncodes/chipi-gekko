pub mod cpu;

mod gekko_decoder {
    include!(concat!(env!("OUT_DIR"), "/gekko.rs"));
}

mod dsp_decoder {
    include!(concat!(env!("OUT_DIR"), "/dsp.rs"));
}

pub use gekko_decoder::GekkoInstruction;
pub use dsp_decoder::DspInstruction;

use std::env;
use std::fs;
use std::process;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Dol,  // PowerPC/Gekko (32-bit)
    Dsp,  // GameCube DSP (16-bit)
}

fn parse_offset(s: &str) -> Result<usize, std::num::ParseIntError> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        usize::from_str_radix(hex, 16)
    } else {
        s.parse()
    }
}

fn print_usage(prog: &str) {
    eprintln!("usage: {} [-m <mode>] <file.bin> [offset]", prog);
    eprintln!();
    eprintln!("options:");
    eprintln!("  -m <mode>    decoder mode: dol (default) or dsp");
    eprintln!("  <file.bin>   binary file to disassemble");
    eprintln!("  [offset]     optional start offset (hex or decimal)");
}

fn disassemble_dol(data: &[u8], start: usize) {
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

fn disassemble_dsp(data: &[u8], start: usize) {
    let mut offset = start;
    while offset + 2 <= data.len() {
        let word = u16::from_be_bytes(data[offset..offset + 2].try_into().unwrap());
        let addr = (offset / 2) as u32;

        // Prepare buffer for multi-unit instructions
        let mut units = Vec::new();
        let mut temp_offset = offset;
        while temp_offset + 2 <= data.len() && units.len() < 3 {
            units.push(u16::from_be_bytes(data[temp_offset..temp_offset + 2].try_into().unwrap()));
            temp_offset += 2;
        }

        match DspInstruction::decode(&units) {
            Some((instr, consumed)) => {
                let hex_str: Vec<String> = units[..consumed].iter()
                    .map(|w| format!("{:04x}", w))
                    .collect();

                let hex_part = hex_str.join(" ");
                println!("{:04x} {:10} {}", addr, hex_part, instr);
                offset += consumed * 2;
            }
            None => {
                println!("{:04x} {:10} .word     {:#06x}", addr, format!("{:04x}", word), word);
                offset += 2;
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }

    let mut mode = Mode::Dol;
    let mut file_idx = 1;

    if args.len() >= 3 && args[1] == "-m" {
        match args[2].as_str() {
            "dol" => mode = Mode::Dol,
            "dsp" => mode = Mode::Dsp,
            other => {
                eprintln!("error: invalid mode '{}' (must be 'dol' or 'dsp')", other);
                process::exit(1);
            }
        }
        file_idx = 3;
    }

    if args.len() <= file_idx {
        print_usage(&args[0]);
        process::exit(1);
    }

    let data = fs::read(&args[file_idx]).unwrap_or_else(|e| {
        eprintln!("failed to read {}: {}", args[file_idx], e);
        process::exit(1);
    });

    let start = if args.len() > file_idx + 1 {
        parse_offset(&args[file_idx + 1]).unwrap_or_else(|e| {
            eprintln!("invalid offset '{}': {}", args[file_idx + 1], e);
            process::exit(1);
        })
    } else {
        0
    };

    match mode {
        Mode::Dol => {
            if data.len() < start + 4 {
                eprintln!("file too small for offset {:#x}", start);
                process::exit(1);
            }
            disassemble_dol(&data, start);
        }
        Mode::Dsp => {
            if data.len() < start + 2 {
                eprintln!("file too small for offset {:#x}", start);
                process::exit(1);
            }
            disassemble_dsp(&data, start);
        }
    }
}
