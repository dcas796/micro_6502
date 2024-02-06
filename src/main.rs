#![feature(let_chains)]

use std::fs::read;
use crate::decoder::Decoder;
use crate::emulator::Emulator;
use crate::instruction::InstructionRegistry;
use crate::mem::Memory;
use crate::regs::Regs;

mod regs;
mod instruction;
mod mem;
mod decoder;
mod emulator;
mod bus;

fn main() {
    let registry = InstructionRegistry::new();
    let instruction_bytes = read("./examples/fibonacci.bin").unwrap();
    let decoder = Decoder::new(registry, instruction_bytes);
    let memory = Memory::new();
    let mut regs = Regs::new();
    // 0, 1, 1, 2, 3, 5, 8, 13, 21
    regs.x = 13;
    let mut emulator = Emulator::new(decoder, memory, regs);
    emulator.run_until_completion();
    println!("{}", emulator.get_regs());
    println!("{}", emulator.get_memory());
}
