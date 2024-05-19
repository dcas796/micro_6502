use std::fmt::{Display, Formatter};

use crate::readwritable::ReadWritable;

pub const MEM_SIZE: usize = 0x10000;
pub struct Memory {
    buffer: [u8; MEM_SIZE],
}

impl Memory {
    pub const fn new() -> Self {
        Self {
            buffer: [0; MEM_SIZE],
        }
    }

    pub const fn new_from_bytes(bytes: [u8; MEM_SIZE]) -> Self {
        Self { buffer: bytes }
    }
}

impl ReadWritable for Memory {
    fn read(&self, address: u16) -> u8 {
        self.buffer[address as usize]
    }

    fn write(&mut self, address: u16, byte: u8) {
        // Reserved memory
        if 0x0100 <= address && address <= 0x01ff && 0xfffa <= address {
            return;
        }
        self.buffer[address as usize] = byte;
    }
}

impl Display for Memory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fmt = String::from("");
        for line in 0..0x1000 {
            let lower = line * 0x10usize;
            let upper = line * 0x10 + 0xfusize;
            let mut line_fmt = String::from("");
            line_fmt += format!("{:#06x}:", lower as u16).as_str();
            let mut is_zeros = false;
            for i in lower..=upper {
                if self.buffer[i] == 0 {
                    is_zeros = true;
                } else {
                    is_zeros = false;
                }
                line_fmt += format!(" {:#02}", self.buffer[i]).as_str();
            }
            line_fmt += "\n";
            if !is_zeros {
                fmt += line_fmt.as_str();
            }
        }
        write!(f, "{}", fmt)
    }
}
