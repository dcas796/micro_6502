use crate::instruction::{AddressingMode, Instruction, InstructionRegistry};

pub struct Decoder {
    buffer: Vec<u8>,
    ptr: u16,
    registry: InstructionRegistry,
}

impl Decoder {
    pub fn new(registry: InstructionRegistry, bytes: Vec<u8>) -> Self {
        if bytes.len() == 0 {
            panic!("Buffer cannot be empty")
        }
        if bytes.len() > 0x10000 {
            panic!("Buffer cannot be bigger than 0x10000 bytes")
        }
        Self {
            buffer: bytes,
            ptr: 0,
            registry: registry,
        }
    }

    pub fn offset(&self) -> u16 {
        self.ptr
    }

    pub fn seek(&mut self, offset: u16) {
        if offset >= self.buffer.len() as u16 {
            panic!("Cannot seek further than the length of the buffer")
        }
        self.ptr = offset;
    }

    pub fn next(&mut self) -> Option<u8> {
        if self.ptr >= self.buffer.len() as u16 {
            return None;
        }

        let byte = self.buffer[self.ptr as usize];
        self.ptr += 1;
        return Some(byte);
    }

    pub fn next_word(&mut self) -> Option<u16> {
        let lower = self.next()? as u16;
        let higher = self.next()? as u16;

        Some((higher << 8) | lower)
    }

    pub fn decode_next(&mut self) -> Option<Instruction> {
        let byte = self.next()?;
        let mut instruction = self.registry.get_instruction_by_op_code(byte, 0)?;

        match instruction.addressing_mode {
            // No operand
            AddressingMode::Implicit |
            AddressingMode::Accumulator => {}

            // Byte operand
            AddressingMode::Immediate |
            AddressingMode::ZeroPage |
            AddressingMode::ZeroPageX |
            AddressingMode::ZeroPageY |
            AddressingMode::Relative |
            AddressingMode::IndirectX |
            AddressingMode::IndirectY => {
                let operand = self.next()?;
                instruction.operand = operand as u16;
            }

            // Word operand
            AddressingMode::Absolute |
            AddressingMode::AbsoluteX |
            AddressingMode::AbsoluteY |
            AddressingMode::Indirect => {
                let operand = self.next_word()?;
                instruction.operand = operand;
            }
        }

        Some(instruction)
    }

    pub fn decode_all(mut self) -> Vec<Instruction> {
        let mut instructions = vec![];

        while let Some(instruction) = self.decode_next() {
            instructions.push(instruction);
        }

        instructions
    }
}
