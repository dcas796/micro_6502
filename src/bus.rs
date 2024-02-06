use crate::mem::Memory;

pub trait ReadWritable {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, byte: u8);
}

pub struct Bus {
    memory: Memory
}

impl Bus {
    pub const fn new(memory: Memory) -> Self {
        Self {
            memory
        }
    }
}

impl ReadWritable for Bus {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0xffff => self.memory.read(address),
            _ => panic!(),  // Error in Intellij rust plugin
        }
    }

    fn write(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0xffff => self.memory.write(address, byte),
            _ => panic!(),  // Error in Intellij rust plugin
        }
    }
}
