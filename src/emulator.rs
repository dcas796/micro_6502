use bitflags::Flags;
use crate::bus::ReadWritable;
use crate::decoder::Decoder;
use crate::instruction::{AddressingMode, Instruction, InstructionName};
use crate::mem::Memory;
use crate::regs::{CpuFlags, Regs};

pub struct Emulator {
    decoder: Decoder,
    memory: Memory,
    regs: Regs,
}

impl Emulator {
    pub fn new(decoder: Decoder, memory: Memory, regs: Regs) -> Self {
        Self {
            decoder,
            memory,
            regs
        }
    }

    pub fn run_until_completion(&mut self) {
        self.set_pc(0);
        while let Some(instruction) = self.decode_next() {
            println!("Executing instruction: {:#06x}: {}", self.regs.pc, instruction);
            self.execute(instruction);
        }
    }

    pub fn get_memory(&self) -> &Memory {
        &self.memory
    }

    pub fn get_regs(&self) -> &Regs {
        &self.regs
    }

    fn set_pc(&mut self, pc: u16) {
        println!("PC -> {:#06x}", pc);
        self.regs.pc = pc;
        self.decoder.seek(pc);
    }

    fn inc_pc(&mut self) {
        self.set_pc(self.regs.pc + 1);
    }

    fn decode_next(&mut self) -> Option<Instruction> {
        let ins = self.decoder.decode_next();
        self.regs.pc = self.decoder.offset();
        ins
    }

    fn get_absolute_address(&self, mode: AddressingMode, address: u16) -> u16 {
        match mode {
            AddressingMode::Implicit => panic!("Cannot get an address when addressing_mode=Implicit"),
            AddressingMode::Accumulator => panic!("Cannot get an address when addressing_mode=Accumulator"),
            AddressingMode::Immediate => panic!("Cannot get an address when addressing_mode=Immediate"),
            AddressingMode::ZeroPage => address,
            AddressingMode::ZeroPageX => address + self.regs.x as u16,
            AddressingMode::ZeroPageY => address + self.regs.y as u16,
            AddressingMode::Relative => self.regs.pc + address,
            AddressingMode::Absolute => address,
            AddressingMode::AbsoluteX => address + self.regs.x as u16,
            AddressingMode::AbsoluteY => address + self.regs.y as u16,
            AddressingMode::Indirect => {
                let mut addr = self.read_byte(AddressingMode::Absolute, address) as u16;
                addr |= (self.read_byte(AddressingMode::Absolute, address + 1) as u16) << 8;
                addr
            }
            AddressingMode::IndirectX => {
                let address = address + self.regs.x as u16;
                let mut addr = self.read_byte(AddressingMode::Absolute, address) as u16;
                addr |= (self.read_byte(AddressingMode::Absolute, address + 1) as u16) << 8;
                addr
            }
            AddressingMode::IndirectY => {
                let mut addr = self.read_byte(AddressingMode::Absolute, address) as u16;
                addr |= (self.read_byte(AddressingMode::Absolute, address + 1) as u16) << 8;
                addr + self.regs.y as u16
            }
        }
    }

    fn read_byte(&self, mode: AddressingMode, address: u16) -> u8 {
        if mode == AddressingMode::Accumulator {
            return self.regs.a;
        }
        if mode == AddressingMode::Immediate {
            return address as u8;
        }
        let absolute_address = self.get_absolute_address(mode, address);
        self.memory.read(absolute_address)
    }

    fn write_byte(&mut self, mode: AddressingMode, address: u16, byte: u8) {
        if mode == AddressingMode::Accumulator {
            self.regs.a = byte;
            return;
        }
        let absolute_address = self.get_absolute_address(mode, address);
        self.memory.write(absolute_address, byte);
    }

    fn push(&mut self, byte: u8) {
        if self.regs.sp == 0 {
            panic!("Ran out of stack");
        }
        self.memory.write_to_stack(self.regs.sp, byte);
        
        self.regs.sp -= 1;
        println!("sp - 1 = {:#04x}", self.regs.sp);
    }

    fn pull(&mut self) -> u8 {
        if self.regs.sp == 0xff {
            panic!("Cannot pull from an empty stack");
        }
        self.regs.sp += 1;
        let byte = self.memory.read_from_stack(self.regs.sp);
        byte
    }

    fn push_pc(&mut self, offset: u16) {
        let pc = self.regs.pc + offset;
        self.push((pc >> 8) as u8);
        self.push((pc & 0xff) as u8);
    }

    fn pull_pc(&mut self, offset: u16) -> u16 {
        let mut pc = self.pull() as u16;
        pc |= (self.pull() as u16) << 8;
        pc += offset;
        println!("PC <- {:#04x}", pc);
        pc
    }

    fn push_flags(&mut self) {
        self.regs.flags.insert(CpuFlags::BREAK);
        self.push(self.regs.flags.bits());
    }

    fn pull_flags(&mut self) {
        let contains_break = self.regs.flags.contains(CpuFlags::BREAK);
        self.regs.flags = CpuFlags::from_bits(self.pull()).unwrap();
        if contains_break {
            self.regs.flags.insert(CpuFlags::BREAK);
        } else {
            self.regs.flags.remove(CpuFlags::BREAK);
        }
    }

    fn interrupt(&mut self) {
        println!("Cannot handle interrupts. Ignoring...");
    }

    fn add(&mut self, a: u8, b: u8) -> u8 {
        let mut result = a;
        if (result as usize + b as usize + self.carry() as usize) > 0xff {
            self.regs.flags.insert(CpuFlags::CARRY);
            self.regs.flags.insert(CpuFlags::OVERFLOW);
            result += b + self.carry() - 0xff;
        } else {
            result += b + self.carry();
        }

        self.set_zero_or_neg(result);

        result
    }

    fn sub(&mut self, a: u8, b: u8) -> u8 {
        let mut result = a;
        if (result as isize - b as isize - self.carry() as isize) < 0 {
            self.regs.flags.insert(CpuFlags::CARRY);
            result = 0xff - (result + b + self.carry());
        } else {
            result -= b + self.carry();
        }

        self.set_zero_or_neg(result);

        result
    }

    fn shl(&mut self, a: u8) -> u8 {
        let result = a << 1;
        if a >> 7 == 1 {
            self.regs.flags.insert(CpuFlags::CARRY);
        } else {
            self.regs.flags.remove(CpuFlags::CARRY);
        }
        self.set_zero_or_neg(a);
        result
    }

    fn shr(&mut self, a: u8) -> u8 {
        let result = a >> 1;
        if a & 1 == 1 {
            self.regs.flags.insert(CpuFlags::CARRY);
        } else {
            self.regs.flags.remove(CpuFlags::CARRY);
        }
        self.set_zero_or_neg(a);
        result
    }

    fn rol(&mut self, a: u8) -> u8 {
        let mut result = self.shl(a);
        if self.regs.flags.contains(CpuFlags::CARRY) {
            result |= 1;
        }
        self.set_zero_or_neg(result);
        result
    }

    fn ror(&mut self, a: u8) -> u8 {
        let mut result = self.shr(a);
        if self.regs.flags.contains(CpuFlags::CARRY) {
            result |= 1 << 7;
        }
        self.set_zero_or_neg(result);
        result
    }

    fn set_zero_or_neg(&mut self, result: u8) {
        if result == 0 {
            self.regs.flags.insert(CpuFlags::ZERO);
        } else {
            self.regs.flags.remove(CpuFlags::ZERO);
        }
        if self.regs.x >= 0b1000_0000 {
            self.regs.flags.insert(CpuFlags::NEG);
        } else {
            self.regs.flags.remove(CpuFlags::NEG);
        }
    }

    fn carry(&self) -> u8 {
        if self.regs.flags.contains(CpuFlags::CARRY) {
            1
        } else {
            0
        }
    }

    fn execute(&mut self, ins: Instruction) {
        match ins.name {
            InstructionName::lda => {
                self.regs.a = self.read_byte(ins.addressing_mode, ins.operand);
                self.set_zero_or_neg(self.regs.a);
            }
            InstructionName::ldx => {
                self.regs.x = self.read_byte(ins.addressing_mode, ins.operand);
                self.set_zero_or_neg(self.regs.x);
            }
            InstructionName::ldy => {
                self.regs.y = self.read_byte(ins.addressing_mode, ins.operand);
                self.set_zero_or_neg(self.regs.y);
            }
            InstructionName::sta => {
                self.write_byte(ins.addressing_mode, ins.operand, self.regs.a);
            }
            InstructionName::stx => {
                self.write_byte(ins.addressing_mode, ins.operand, self.regs.x);
            }
            InstructionName::sty => {
                self.write_byte(ins.addressing_mode, ins.operand, self.regs.y);
            }

            InstructionName::tax => {
                self.regs.x = self.regs.a;
                self.set_zero_or_neg(self.regs.x);
            }
            InstructionName::tay => {
                self.regs.y = self.regs.a;
                self.set_zero_or_neg(self.regs.y);
            }
            InstructionName::txa => {
                self.regs.a = self.regs.x;
                self.set_zero_or_neg(self.regs.a);
            }
            InstructionName::tya => {
                self.regs.a = self.regs.y;
                self.set_zero_or_neg(self.regs.a);
            }

            InstructionName::tsx => {
                self.regs.x = self.regs.sp;
                self.set_zero_or_neg(self.regs.x);
            }
            InstructionName::txs => {
                self.regs.sp = self.regs.x;
            }
            InstructionName::pha => {
                self.push(self.regs.a);
            }
            InstructionName::php => {
                self.push_flags();
            }
            InstructionName::pla => {
                self.regs.a = self.pull();
                self.set_zero_or_neg(self.regs.a);
            }
            InstructionName::plp => {
                self.pull_flags()
            }

            InstructionName::and => {
                self.regs.a = self.regs.a & self.read_byte(ins.addressing_mode, ins.operand);
                self.set_zero_or_neg(self.regs.a);
            }
            InstructionName::eor => {
                self.regs.a = self.regs.a ^ self.read_byte(ins.addressing_mode, ins.operand);
                self.set_zero_or_neg(self.regs.a);
            }
            InstructionName::ora => {
                self.regs.a = self.regs.a | self.read_byte(ins.addressing_mode, ins.operand);
                self.set_zero_or_neg(self.regs.a);
            }
            InstructionName::bit => {
                let byte = self.read_byte(ins.addressing_mode, ins.operand);
                if byte >> 7 == 1 {
                    self.regs.flags.insert(CpuFlags::NEG);
                }
                if (byte >> 6) & 1 == 1 {
                    self.regs.flags.insert(CpuFlags::OVERFLOW);
                }
                let and = self.regs.a & byte;
                if and == 0 {
                    self.regs.flags.insert(CpuFlags::ZERO);
                } else {
                    self.regs.flags.remove(CpuFlags::ZERO);
                }
            }

            InstructionName::adc => {
                let byte = self.read_byte(ins.addressing_mode, ins.operand);
                self.regs.a = self.add(self.regs.a, byte);
            }
            InstructionName::sbc => {
                let byte = self.read_byte(ins.addressing_mode, ins.operand);
                self.regs.a = self.sub(self.regs.a, byte);
            }
            InstructionName::cmp => {
                let byte = self.read_byte(ins.addressing_mode, ins.operand);
                _ = self.sub(self.regs.a, byte);
            }
            InstructionName::cpx => {
                let byte = self.read_byte(ins.addressing_mode, ins.operand);
                _ = self.sub(self.regs.x, byte);
            }
            InstructionName::cpy => {
                let byte = self.read_byte(ins.addressing_mode, ins.operand);
                _ = self.sub(self.regs.y, byte);
            }

            InstructionName::inc => {
                let mut byte = self.read_byte(ins.addressing_mode, ins.operand);
                byte = self.add(byte, 1);
                self.write_byte(ins.addressing_mode, ins.operand, byte);
            }
            InstructionName::inx => {
                self.regs.x = self.add(self.regs.x, 1);
            }
            InstructionName::iny => {
                self.regs.y = self.add(self.regs.y, 1);
            }
            InstructionName::dec => {
                let mut byte = self.read_byte(ins.addressing_mode, ins.operand);
                let has_carry = self.regs.flags.contains(CpuFlags::CARRY);
                byte = self.sub(byte, 1);
                self.write_byte(ins.addressing_mode, ins.operand, byte);
                if has_carry {
                    self.regs.flags.insert(CpuFlags::CARRY);
                } else {
                    self.regs.flags.remove(CpuFlags::CARRY);
                }
            }
            InstructionName::dex => {
                let has_carry = self.regs.flags.contains(CpuFlags::CARRY);
                self.regs.x = self.sub(self.regs.x, 1);
                if has_carry {
                    self.regs.flags.insert(CpuFlags::CARRY);
                } else {
                    self.regs.flags.remove(CpuFlags::CARRY);
                }
            }
            InstructionName::dey => {
                let has_carry = self.regs.flags.contains(CpuFlags::CARRY);
                self.regs.y = self.sub(self.regs.y, 1);
                if has_carry {
                    self.regs.flags.insert(CpuFlags::CARRY);
                } else {
                    self.regs.flags.remove(CpuFlags::CARRY);
                }
            }

            InstructionName::asl => {
                let mut byte = self.read_byte(ins.addressing_mode, ins.operand);
                byte = self.shl(byte);
                self.write_byte(ins.addressing_mode, ins.operand, byte);
            }
            InstructionName::lsr => {
                let mut byte = self.read_byte(ins.addressing_mode, ins.operand);
                byte = self.shr(byte);
                self.write_byte(ins.addressing_mode, ins.operand, byte);
            }
            InstructionName::rol => {
                let mut byte = self.read_byte(ins.addressing_mode, ins.operand);
                byte = self.rol(byte);
                self.write_byte(ins.addressing_mode, ins.operand, byte);
            }
            InstructionName::ror => {
                let mut byte = self.read_byte(ins.addressing_mode, ins.operand);
                byte = self.ror(byte);
                self.write_byte(ins.addressing_mode, ins.operand, byte);
            }

            InstructionName::jmp => {
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }
            InstructionName::jsr => {
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.push_pc(0);
                self.set_pc(addr);
                println!("x = {}", self.regs.x);
            }
            InstructionName::rts => {
                let pc = self.pull_pc(0);
                self.set_pc(pc);
            }

            InstructionName::bcc => {
                if self.regs.flags.contains(CpuFlags::CARRY) { return; }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }
            InstructionName::bcs => {
                if !self.regs.flags.contains(CpuFlags::CARRY) { return; }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }
            InstructionName::beq => {
                if !self.regs.flags.contains(CpuFlags::ZERO) { return; }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }
            InstructionName::bmi => {
                if !self.regs.flags.contains(CpuFlags::NEG) { return; }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }
            InstructionName::bne => {
                if self.regs.flags.contains(CpuFlags::ZERO) { return; }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }
            InstructionName::bpl => {
                if !self.regs.flags.contains(CpuFlags::NEG) { return; }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }
            InstructionName::bvc => {
                if self.regs.flags.contains(CpuFlags::OVERFLOW) { return; }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }
            InstructionName::bvs => {
                if !self.regs.flags.contains(CpuFlags::OVERFLOW) { return; }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }

            InstructionName::clc => {
                self.regs.flags.remove(CpuFlags::CARRY);
            }
            InstructionName::cld => {
                self.regs.flags.remove(CpuFlags::DEC_MODE);
            }
            InstructionName::cli => {
                self.regs.flags.remove(CpuFlags::INT_DISABLE);
            }
            InstructionName::clv => {
                self.regs.flags.remove(CpuFlags::OVERFLOW);
            }
            InstructionName::sec => {
                self.regs.flags.insert(CpuFlags::CARRY);
            }
            InstructionName::sed => {
                self.regs.flags.insert(CpuFlags::DEC_MODE);
            }
            InstructionName::sei => {
                self.regs.flags.insert(CpuFlags::INT_DISABLE);
            }

            InstructionName::brk => {
                self.interrupt();
                let ret_addr = self.regs.pc + 2;
                self.push((ret_addr >> 8) as u8);
                self.push((ret_addr & 0xff) as u8);
                self.push(self.regs.flags.bits());
            }
            InstructionName::nop => {}
            InstructionName::rti => {
                self.pull_flags();
                let pc = self.pull_pc(0);
                self.set_pc(pc);
            }
        }
    }
}
