use clap::Parser;

use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    path::PathBuf,
    str::FromStr,
};

use micro_6502::decoder::Decoder;
use micro_6502::emulator::Emulator;
use micro_6502::instruction::InstructionRegistry;
use micro_6502::mem::{Memory, MEM_SIZE};
use micro_6502::regs::{CpuFlags, Regs};
use std::fs::read;

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

#[derive(Parser)]
pub struct Args {
    /// The path to the binary to execute
    pub path: PathBuf,
    /// Initialize memory with the file provided
    #[arg(long, default_value = None)]
    pub memory: Option<PathBuf>,
    /// Initialize the CPU registers
    /// Example: --regs x=3,y=2
    #[arg(long, default_value_t)]
    pub regs: RegsArg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegsArg {
    pub regs: Regs,
}

impl Default for RegsArg {
    fn default() -> Self {
        Self { regs: Regs::new() }
    }
}

impl Deref for RegsArg {
    type Target = Regs;
    fn deref(&self) -> &Self::Target {
        &self.regs
    }
}

impl DerefMut for RegsArg {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.regs
    }
}

impl FromStr for RegsArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reg_key_value = s.split(',');
        let reg_key_value_tuple: Vec<(&str, u16)> = reg_key_value
            .map(|kv_str| {
                let kv: Vec<&str> = kv_str.split('=').collect();
                if kv.len() == 2 {
                    Ok((
                        kv[0],
                        kv[1]
                            .parse()
                            .map_err(|_| format!("Not a valid u16: {kv_str}"))?,
                    ))
                } else {
                    Err(format!("Cannot parse register argument: {}", kv_str))
                }
            })
            .collect::<Result<Vec<_>, String>>()?;

        let mut regs = Regs::new();
        for (key, value) in reg_key_value_tuple {
            match key {
                "pc" => regs.pc = value,
                "sp" => regs.sp = value as u8,
                "a" => regs.a = value as u8,
                "x" => regs.x = value as u8,
                "y" => regs.y = value as u8,
                "flags" => regs.flags = CpuFlags::from_bits(value as u8).unwrap(),
                _ => return Err(format!("Unknown register: {key}")),
            }
        }

        Ok(Self { regs })
    }
}

impl Display for RegsArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "pc={},sp={},a={},x={},y={},flags={}",
            self.pc,
            self.sp,
            self.a,
            self.x,
            self.y,
            self.flags.bits()
        )
    }
}
