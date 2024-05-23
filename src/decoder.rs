use crate::instruction::{AddressingMode, Instruction, InstructionRegistry};

pub struct Decoder {
    registry: InstructionRegistry,
    next_byte: Box<dyn FnMut() -> u8>,
}

impl Decoder {
    pub fn new(next_byte: Box<dyn FnMut() -> u8>) -> Self {
        Self {
            registry: InstructionRegistry::new(),
            next_byte,
        }
    }

    pub fn next_word(&mut self) -> u16 {
        let lower = (self.next_byte)() as u16;
        let higher = (self.next_byte)() as u16;

        (higher << 8) | lower
    }

    pub fn decode_next(&mut self) -> Instruction {
        let byte = (self.next_byte)();
        let mut instruction = self
            .registry
            .get_instruction_by_op_code(byte, 0)
            .expect(format!("Cannot read op code {:#04x}", byte).as_str());

        match instruction.addressing_mode {
            // No operand
            AddressingMode::Implicit | AddressingMode::Accumulator => {}

            // Byte operand
            AddressingMode::Immediate
            | AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::Relative
            | AddressingMode::IndirectX
            | AddressingMode::IndirectY => {
                instruction.operand = (self.next_byte)() as u16;
            }

            // Word operand
            AddressingMode::Absolute
            | AddressingMode::AbsoluteX
            | AddressingMode::AbsoluteY
            | AddressingMode::Indirect => {
                instruction.operand = self.next_word();
            }
        }

        instruction
    }
}
