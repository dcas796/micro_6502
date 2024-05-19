#![feature(let_chains)]

use clap::Parser;

use crate::args::Args;
use crate::decoder::Decoder;
use crate::emulator::Emulator;
use crate::instruction::InstructionRegistry;
use crate::mem::{Memory, MEM_SIZE};
use std::fs::read;

mod args;
mod decoder;
mod emulator;
mod instruction;
mod mem;
mod readwritable;
mod regs;

fn main() {
    let mut args = Args::parse();

    let registry = InstructionRegistry::new();
    let instruction_bytes =
        read(&args.path).expect(format!("Cannot find '{}'", args.path.display()).as_str());
    let mut decoder = Decoder::new(registry, instruction_bytes);
    let mut memory = if let Some(memory_path) = args.memory {
        let raw_bytes =
            read(memory_path).expect(format!("Cannot find '{}'", args.path.display()).as_str());
        assert!(
            raw_bytes.len() <= MEM_SIZE,
            "Inputted memory file is larger than {MEM_SIZE} bytes."
        );
        let mut raw_bytes_slice = [0u8; MEM_SIZE];
        raw_bytes_slice[..raw_bytes.len()].clone_from_slice(&raw_bytes);
        Memory::new_from_bytes(raw_bytes_slice)
    } else {
        Memory::new()
    };
    let mut emulator = Emulator::new(&mut decoder, &mut memory, &mut args.regs);
    emulator.run_until_completion();
    println!("{}", emulator.get_regs());
    println!("{}", &memory);
}
