use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::decoder::Decoder;
use crate::instruction::{AddressingMode, Instruction, InstructionName};
use crate::readwritable::ReadWritable;
use crate::regs::{CpuFlags, Regs};

pub const RESET_VEC_LOW_ADDR: u16 = 0xfffc;
pub const RESET_VEC_HIGH_ADDR: u16 = 0xfffd;
pub const IRQ_VEC_LOW_ADDR: u16 = 0xfffe;
pub const IRQ_VEC_HIGH_ADDR: u16 = 0xffff;

pub struct Emulator {
    decoder: Decoder,
    regs: Rc<RefCell<Regs>>,
    bus: Rc<RefCell<Box<dyn ReadWritable>>>,
    stop_signalled: bool,
}

impl Emulator {
    pub fn new(bus: Box<dyn ReadWritable>) -> Self {
        let bus_rc = Rc::new(RefCell::new(bus));
        let regs = Rc::new(RefCell::new(Regs::new()));
        let next_byte = {
            let bus_rc = bus_rc.clone();
            let regs = regs.clone();
            move || {
                let bus = bus_rc.borrow();
                let mut regs = regs.borrow_mut();
                let byte = bus.read(regs.pc);
                regs.pc += 1;
                byte
            }
        };
        let decoder = Decoder::new(Box::new(next_byte));
        Self {
            decoder,
            regs,
            bus: bus_rc,
            stop_signalled: false,
        }
    }

    pub fn run_until_break(&mut self) {
        self.run(|_, _| true)
    }

    pub fn run<F: Fn(&Regs, &dyn ReadWritable) -> bool>(&mut self, on_break: F) {
        self.stop_signalled = false;
        let reset_addr = self.get_reset_addr();
        self.set_pc(reset_addr);
        loop {
            while !self.stop_signalled {
                self.execute_next();
            }
            self.stop_signalled = false;
            if on_break(&*self.get_regs(), &**self.get_bus()) {
                break;
            }
        }
    }

    pub fn get_regs(&self) -> Ref<Regs> {
        self.regs.borrow()
    }

    pub fn get_regs_mut(&mut self) -> RefMut<Regs> {
        self.regs.borrow_mut()
    }

    pub fn get_bus(&self) -> Ref<Box<dyn ReadWritable>> {
        self.bus.borrow()
    }

    pub fn get_bus_mut(&mut self) -> RefMut<Box<dyn ReadWritable>> {
        self.bus.borrow_mut()
    }

    fn set_pc(&mut self, pc: u16) {
        self.get_regs_mut().pc = pc;
    }

    fn read_from_stack(&self) -> u8 {
        self.get_bus().read(self.get_regs().sp as u16 + 0x100)
    }

    fn write_to_stack(&mut self, byte: u8) {
        let addr = self.get_regs().sp as u16 + 0x100;
        self.get_bus_mut().write(addr, byte);
    }

    fn get_reset_addr(&self) -> u16 {
        let low = self.get_bus().read(RESET_VEC_LOW_ADDR) as u16;
        let high = self.get_bus().read(RESET_VEC_HIGH_ADDR) as u16;
        (high << 8) | low
    }

    fn get_irq_addr(&self) -> u16 {
        let low = self.get_bus().read(IRQ_VEC_LOW_ADDR) as u16;
        let high = self.get_bus().read(IRQ_VEC_HIGH_ADDR) as u16;
        (high << 8) | low
    }

    fn decode_next(&mut self) -> Instruction {
        self.decoder.decode_next()
    }

    fn get_absolute_address(&self, mode: AddressingMode, address: u16) -> u16 {
        match mode {
            AddressingMode::Implicit => {
                panic!("Cannot get an address when addressing_mode=Implicit")
            }
            AddressingMode::Accumulator => {
                panic!("Cannot get an address when addressing_mode=Accumulator")
            }
            AddressingMode::Immediate => {
                panic!("Cannot get an address when addressing_mode=Immediate")
            }
            AddressingMode::ZeroPage => address,
            AddressingMode::ZeroPageX => address + self.get_regs().x as u16,
            AddressingMode::ZeroPageY => address + self.get_regs().y as u16,
            AddressingMode::Relative => self.get_regs().pc + address,
            AddressingMode::Absolute => address,
            AddressingMode::AbsoluteX => address + self.get_regs().x as u16,
            AddressingMode::AbsoluteY => address + self.get_regs().y as u16,
            AddressingMode::Indirect => {
                let mut addr = self.read_byte(AddressingMode::Absolute, address) as u16;
                addr |= (self.read_byte(AddressingMode::Absolute, address + 1) as u16) << 8;
                addr
            }
            AddressingMode::IndirectX => {
                let address = address + self.get_regs().x as u16;
                let mut addr = self.read_byte(AddressingMode::Absolute, address) as u16;
                addr |= (self.read_byte(AddressingMode::Absolute, address + 1) as u16) << 8;
                addr
            }
            AddressingMode::IndirectY => {
                let mut addr = self.read_byte(AddressingMode::Absolute, address) as u16;
                addr |= (self.read_byte(AddressingMode::Absolute, address + 1) as u16) << 8;
                addr + self.get_regs().y as u16
            }
        }
    }

    fn read_byte(&self, mode: AddressingMode, address: u16) -> u8 {
        if mode == AddressingMode::Accumulator {
            return self.get_regs().a;
        }
        if mode == AddressingMode::Immediate {
            return address as u8;
        }
        let absolute_address = self.get_absolute_address(mode, address);
        self.get_bus().read(absolute_address)
    }

    fn write_byte(&mut self, mode: AddressingMode, address: u16, byte: u8) {
        if mode == AddressingMode::Accumulator {
            self.get_regs_mut().a = byte;
            return;
        }
        let absolute_address = self.get_absolute_address(mode, address);
        self.get_bus_mut().write(absolute_address, byte);
    }

    fn push(&mut self, byte: u8) {
        if self.get_regs().sp == 0 {
            panic!("Ran out of stack");
        }
        self.write_to_stack(byte);

        self.get_regs_mut().sp -= 1;
    }

    fn pull(&mut self) -> u8 {
        if self.get_regs().sp == 0xff {
            panic!("Cannot pull from an empty stack");
        }
        self.get_regs_mut().sp += 1;
        let byte = self.read_from_stack();
        byte
    }

    fn push_pc(&mut self, offset: u16) {
        let pc = self.get_regs().pc + offset;
        self.push((pc >> 8) as u8);
        self.push((pc & 0xff) as u8);
    }

    fn pull_pc(&mut self, offset: u16) -> u16 {
        let mut pc = self.pull() as u16;
        pc |= (self.pull() as u16) << 8;
        pc += offset;
        pc
    }

    fn push_flags(&mut self) {
        self.get_regs_mut().flags.insert(CpuFlags::BREAK);
        let flags = self.get_regs().flags.bits();
        self.push(flags);
    }

    fn pull_flags(&mut self) {
        let contains_break = self.get_regs().flags.contains(CpuFlags::BREAK);
        self.get_regs_mut().flags = CpuFlags::from_bits(self.pull()).unwrap();
        if contains_break {
            self.get_regs_mut().flags.insert(CpuFlags::BREAK);
        } else {
            self.get_regs_mut().flags.remove(CpuFlags::BREAK);
        }
    }

    fn interrupt(&mut self) {
        self.stop_signalled = true;
        let irq_addr = self.get_irq_addr();
        self.set_pc(irq_addr);
    }

    fn add(&mut self, a: u8, b: u8) -> u8 {
        let mut result = a;
        if (result as usize + b as usize + self.carry() as usize) > 0xff {
            self.get_regs_mut().flags.insert(CpuFlags::CARRY);
            self.get_regs_mut().flags.insert(CpuFlags::OVERFLOW);
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
            self.get_regs_mut().flags.insert(CpuFlags::CARRY);
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
            self.get_regs_mut().flags.insert(CpuFlags::CARRY);
        } else {
            self.get_regs_mut().flags.remove(CpuFlags::CARRY);
        }
        self.set_zero_or_neg(a);
        result
    }

    fn shr(&mut self, a: u8) -> u8 {
        let result = a >> 1;
        if a & 1 == 1 {
            self.get_regs_mut().flags.insert(CpuFlags::CARRY);
        } else {
            self.get_regs_mut().flags.remove(CpuFlags::CARRY);
        }
        self.set_zero_or_neg(a);
        result
    }

    fn rol(&mut self, a: u8) -> u8 {
        let mut result = self.shl(a);
        if self.get_regs().flags.contains(CpuFlags::CARRY) {
            result |= 1;
        }
        self.set_zero_or_neg(result);
        result
    }

    fn ror(&mut self, a: u8) -> u8 {
        let mut result = self.shr(a);
        if self.get_regs().flags.contains(CpuFlags::CARRY) {
            result |= 1 << 7;
        }
        self.set_zero_or_neg(result);
        result
    }

    fn set_zero_or_neg(&mut self, result: u8) {
        if result == 0 {
            self.get_regs_mut().flags.insert(CpuFlags::ZERO);
        } else {
            self.get_regs_mut().flags.remove(CpuFlags::ZERO);
        }
        if self.get_regs().x >= 0b1000_0000 {
            self.get_regs_mut().flags.insert(CpuFlags::NEG);
        } else {
            self.get_regs_mut().flags.remove(CpuFlags::NEG);
        }
    }

    fn carry(&self) -> u8 {
        if self.get_regs().flags.contains(CpuFlags::CARRY) {
            1
        } else {
            0
        }
    }

    fn execute_next(&mut self) {
        let instruction = self.decode_next();
        self.execute(instruction);
    }

    fn execute(&mut self, ins: Instruction) {
        match ins.name {
            InstructionName::lda => {
                self.get_regs_mut().a = self.read_byte(ins.addressing_mode, ins.operand);
                let a = self.get_regs().a;
                self.set_zero_or_neg(a);
            }
            InstructionName::ldx => {
                self.get_regs_mut().x = self.read_byte(ins.addressing_mode, ins.operand);
                let x = self.get_regs().x;
                self.set_zero_or_neg(x);
            }
            InstructionName::ldy => {
                self.get_regs_mut().y = self.read_byte(ins.addressing_mode, ins.operand);
                let y = self.get_regs().y;
                self.set_zero_or_neg(y);
            }
            InstructionName::sta => {
                let a = self.get_regs().a;
                self.write_byte(ins.addressing_mode, ins.operand, a);
            }
            InstructionName::stx => {
                let x = self.get_regs().x;
                self.write_byte(ins.addressing_mode, ins.operand, x);
            }
            InstructionName::sty => {
                let y = self.get_regs().y;
                self.write_byte(ins.addressing_mode, ins.operand, y);
            }

            InstructionName::tax => {
                let a = self.get_regs().a;
                self.get_regs_mut().x = a;
                self.set_zero_or_neg(a);
            }
            InstructionName::tay => {
                let a = self.get_regs().a;
                self.get_regs_mut().y = a;
                self.set_zero_or_neg(a);
            }
            InstructionName::txa => {
                let x = self.get_regs().x;
                self.get_regs_mut().a = x;
                self.set_zero_or_neg(x);
            }
            InstructionName::tya => {
                let y = self.get_regs().y;
                self.get_regs_mut().a = y;
                self.set_zero_or_neg(y);
            }

            InstructionName::tsx => {
                let sp = self.get_regs().sp;
                self.get_regs_mut().x = sp;
                self.set_zero_or_neg(sp);
            }
            InstructionName::txs => {
                let x = self.get_regs().x;
                self.get_regs_mut().sp = x;
                self.set_zero_or_neg(x);
            }
            InstructionName::pha => {
                let a = self.get_regs().a;
                self.push(a);
            }
            InstructionName::php => {
                self.push_flags();
            }
            InstructionName::pla => {
                self.get_regs_mut().a = self.pull();
                let a = self.get_regs().a;
                self.set_zero_or_neg(a);
            }
            InstructionName::plp => self.pull_flags(),

            InstructionName::and => {
                let result = self.get_regs().a & self.read_byte(ins.addressing_mode, ins.operand);
                self.get_regs_mut().a = result;
                self.set_zero_or_neg(result);
            }
            InstructionName::eor => {
                let result = self.get_regs().a ^ self.read_byte(ins.addressing_mode, ins.operand);
                self.get_regs_mut().a = result;
                self.set_zero_or_neg(result);
            }
            InstructionName::ora => {
                let result = self.get_regs().a | self.read_byte(ins.addressing_mode, ins.operand);
                self.get_regs_mut().a = result;
                self.set_zero_or_neg(result);
            }
            InstructionName::bit => {
                let byte = self.read_byte(ins.addressing_mode, ins.operand);
                if byte >> 7 == 1 {
                    self.get_regs_mut().flags.insert(CpuFlags::NEG);
                }
                if (byte >> 6) & 1 == 1 {
                    self.get_regs_mut().flags.insert(CpuFlags::OVERFLOW);
                }
                let and = self.get_regs().a & byte;
                if and == 0 {
                    self.get_regs_mut().flags.insert(CpuFlags::ZERO);
                } else {
                    self.get_regs_mut().flags.remove(CpuFlags::ZERO);
                }
            }

            InstructionName::adc => {
                let byte = self.read_byte(ins.addressing_mode, ins.operand);
                let a = self.get_regs().a;
                self.get_regs_mut().a = self.add(a, byte);
            }
            InstructionName::sbc => {
                let byte = self.read_byte(ins.addressing_mode, ins.operand);
                let a = self.get_regs().a;
                self.get_regs_mut().a = self.sub(a, byte);
            }
            InstructionName::cmp => {
                let byte = self.read_byte(ins.addressing_mode, ins.operand);
                let a = self.get_regs().a;
                _ = self.sub(a, byte);
            }
            InstructionName::cpx => {
                let byte = self.read_byte(ins.addressing_mode, ins.operand);
                let x = self.get_regs().x;
                _ = self.sub(x, byte);
            }
            InstructionName::cpy => {
                let byte = self.read_byte(ins.addressing_mode, ins.operand);
                let y = self.get_regs().y;
                _ = self.sub(y, byte);
            }

            InstructionName::inc => {
                let mut byte = self.read_byte(ins.addressing_mode, ins.operand);
                byte = self.add(byte, 1);
                self.write_byte(ins.addressing_mode, ins.operand, byte);
            }
            InstructionName::inx => {
                let x = self.get_regs().x;
                self.get_regs_mut().x = self.add(x, 1);
            }
            InstructionName::iny => {
                let y = self.get_regs().y;
                self.get_regs_mut().y = self.add(y, 1);
            }
            InstructionName::dec => {
                let mut byte = self.read_byte(ins.addressing_mode, ins.operand);
                let has_carry = self.get_regs().flags.contains(CpuFlags::CARRY);
                byte = self.sub(byte, 1);
                self.write_byte(ins.addressing_mode, ins.operand, byte);
                if has_carry {
                    self.get_regs_mut().flags.insert(CpuFlags::CARRY);
                } else {
                    self.get_regs_mut().flags.remove(CpuFlags::CARRY);
                }
            }
            InstructionName::dex => {
                let has_carry = self.get_regs().flags.contains(CpuFlags::CARRY);
                let x = self.get_regs().x;
                self.get_regs_mut().x = self.sub(x, 1);
                if has_carry {
                    self.get_regs_mut().flags.insert(CpuFlags::CARRY);
                } else {
                    self.get_regs_mut().flags.remove(CpuFlags::CARRY);
                }
            }
            InstructionName::dey => {
                let has_carry = self.get_regs().flags.contains(CpuFlags::CARRY);
                let y = self.get_regs().y;
                self.get_regs_mut().y = self.sub(y, 1);
                if has_carry {
                    self.get_regs_mut().flags.insert(CpuFlags::CARRY);
                } else {
                    self.get_regs_mut().flags.remove(CpuFlags::CARRY);
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
            }
            InstructionName::rts => {
                let pc = self.pull_pc(0);
                self.set_pc(pc);
            }

            InstructionName::bcc => {
                if self.get_regs().flags.contains(CpuFlags::CARRY) {
                    return;
                }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }
            InstructionName::bcs => {
                if !self.get_regs().flags.contains(CpuFlags::CARRY) {
                    return;
                }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }
            InstructionName::beq => {
                if !self.get_regs().flags.contains(CpuFlags::ZERO) {
                    return;
                }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }
            InstructionName::bmi => {
                if !self.get_regs().flags.contains(CpuFlags::NEG) {
                    return;
                }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }
            InstructionName::bne => {
                if self.get_regs().flags.contains(CpuFlags::ZERO) {
                    return;
                }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }
            InstructionName::bpl => {
                if !self.get_regs().flags.contains(CpuFlags::NEG) {
                    return;
                }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }
            InstructionName::bvc => {
                if self.get_regs().flags.contains(CpuFlags::OVERFLOW) {
                    return;
                }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }
            InstructionName::bvs => {
                if !self.get_regs().flags.contains(CpuFlags::OVERFLOW) {
                    return;
                }
                let addr = self.get_absolute_address(ins.addressing_mode, ins.operand);
                self.set_pc(addr);
            }

            InstructionName::clc => {
                self.get_regs_mut().flags.remove(CpuFlags::CARRY);
            }
            InstructionName::cld => {
                self.get_regs_mut().flags.remove(CpuFlags::DEC_MODE);
            }
            InstructionName::cli => {
                self.get_regs_mut().flags.remove(CpuFlags::INT_DISABLE);
            }
            InstructionName::clv => {
                self.get_regs_mut().flags.remove(CpuFlags::OVERFLOW);
            }
            InstructionName::sec => {
                self.get_regs_mut().flags.insert(CpuFlags::CARRY);
            }
            InstructionName::sed => {
                self.get_regs_mut().flags.insert(CpuFlags::DEC_MODE);
            }
            InstructionName::sei => {
                self.get_regs_mut().flags.insert(CpuFlags::INT_DISABLE);
            }

            InstructionName::brk => {
                if !self.get_regs().flags.contains(CpuFlags::INT_DISABLE) {
                    self.interrupt();
                    let ret_addr = self.get_regs().pc + 2;
                    self.push((ret_addr >> 8) as u8);
                    self.push((ret_addr & 0xff) as u8);
                    let flags = self.get_regs().flags.bits();
                    self.push(flags);
                }
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
