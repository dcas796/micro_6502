use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use strum_macros::Display;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Display)]
pub enum InstructionName {
    // Load/Store Operations
    lda, ldx, ldy, sta, stx, sty,
    // Register Transfers
    tax, tay, txa, tya,
    // Stack Operations
    tsx, txs, pha, php, pla, plp,
    // Logical
    and, eor, ora, bit,
    // Arithmetic
    adc, sbc, cmp, cpx, cpy,
    // Increments & Decrements
    inc, inx, iny, dec, dex, dey,
    // Shifts
    asl, lsr, rol, ror,
    // Jumps & Calls
    jmp, jsr, rts,
    // Branches
    bcc, bcs, beq, bmi, bne, bpl, bvc, bvs,
    // Status Flag Changes
    clc, cld, cli, clv, sec, sed, sei,
    // System Functions
    brk, nop, rti,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Display)]
pub enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
}

#[derive(Debug, Copy, Clone)]
pub struct Instruction {
    pub name: InstructionName,
    pub addressing_mode: AddressingMode,
    pub op_code: u8,
    pub operand: u16,
}

impl Instruction {
    fn new(name: InstructionName, addressing_mode: AddressingMode, op_code: u8, operand: u16) -> Self {
        Self {
            name,
            addressing_mode,
            op_code,
            operand
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.addressing_mode {
            AddressingMode::Implicit |
            AddressingMode::Accumulator => {
                write!(f, "{}", self.name)
            }
            AddressingMode::Immediate => {
                write!(f, "{} #${:02x}", self.name, self.operand)
            }
            AddressingMode::Relative |
            AddressingMode::ZeroPage => {
                write!(f, "{} ${:02x}", self.name, self.operand)
            }
            AddressingMode::ZeroPageX => {
                write!(f, "{} ${:02x},X", self.name, self.operand)
            }
            AddressingMode::ZeroPageY => {
                write!(f, "{} ${:02x},Y", self.name, self.operand)
            }
            AddressingMode::Absolute => {
                write!(f, "{} ${:04x}", self.name, self.operand)
            }
            AddressingMode::AbsoluteX => {
                write!(f, "{} ${:04x},X", self.name, self.operand)
            }
            AddressingMode::AbsoluteY => {
                write!(f, "{} ${:04x},Y", self.name, self.operand)
            }
            AddressingMode::Indirect => {
                write!(f, "{} (${:04x})", self.name, self.operand)
            }
            AddressingMode::IndirectX => {
                write!(f, "{} (${:02x},X)", self.name, self.operand)
            }
            AddressingMode::IndirectY => {
                write!(f, "{} (${:02x}),Y", self.name, self.operand)
            }
        }
    }
}

#[derive(Debug)]
pub struct InstructionBuilder {
    pub name: InstructionName,
    addressing_modes: HashMap<AddressingMode, u8>
}

impl InstructionBuilder {
    fn new(name: InstructionName) -> Self {
        Self {
            name,
            addressing_modes: HashMap::new(),
        }
    }
    
    pub fn get_modes(&self) -> &HashMap<AddressingMode, u8> {
        &self.addressing_modes
    }

    fn add_mode(mut self, addressing_mode: AddressingMode, op_code: u8) -> Self {
        _ = self.addressing_modes.insert(addressing_mode, op_code);
        self
    }

    pub fn build(&self, addressing_mode: AddressingMode, operand: u16) -> Option<Instruction> {
        let op_code = self.addressing_modes.get(&addressing_mode)?.clone();
        Some(Instruction::new(self.name, addressing_mode, op_code, operand))
    }

    fn imp(self, op_code: u8) -> Self { self.add_mode(AddressingMode::Implicit, op_code) }
    fn acc(self, op_code: u8) -> Self { self.add_mode(AddressingMode::Accumulator, op_code) }
    fn imm(self, op_code: u8) -> Self { self.add_mode(AddressingMode::Immediate, op_code) }
    fn zp(self, op_code: u8) -> Self { self.add_mode(AddressingMode::ZeroPage, op_code) }
    fn zpx(self, op_code: u8) -> Self { self.add_mode(AddressingMode::ZeroPageX, op_code) }
    fn zpy(self, op_code: u8) -> Self { self.add_mode(AddressingMode::ZeroPageY, op_code) }
    fn rel(self, op_code: u8) -> Self { self.add_mode(AddressingMode::Relative, op_code) }
    fn abs(self, op_code: u8) -> Self { self.add_mode(AddressingMode::Absolute, op_code) }
    fn absx(self, op_code: u8) -> Self { self.add_mode(AddressingMode::AbsoluteX, op_code) }
    fn absy(self, op_code: u8) -> Self { self.add_mode(AddressingMode::AbsoluteY, op_code) }
    fn ind(self, op_code: u8) -> Self { self.add_mode(AddressingMode::Indirect, op_code) }
    fn indx(self, op_code: u8) -> Self { self.add_mode(AddressingMode::IndirectX, op_code) }
    fn indy(self, op_code: u8) -> Self { self.add_mode(AddressingMode::IndirectY, op_code) }
}

const NUM_INSTRUCTIONS: usize = 56;
pub struct InstructionRegistry {
    pub all_instructions: [InstructionBuilder; NUM_INSTRUCTIONS],
}

impl InstructionRegistry {
    pub fn new() -> Self {
        Self {
            all_instructions: Self::get_all_instructions(),
        }
    }

    pub fn get_instruction_by_name(&self, name: InstructionName) -> &InstructionBuilder {
        if let Some(builder) = self.all_instructions.iter().find(|builder| {
            builder.name == name
        }) {
            builder
        } else {
            panic!("Could not get instruction by name {}", name)
        }
    }

    pub fn get_instruction_by_op_code(&self, op_code: u8, operand: u16) -> Option<Instruction> {
        let mut name: Option<InstructionName> = None;
        let mut addr_mode: Option<AddressingMode> = None;

        for ins in &self.all_instructions {
            for (mode, mode_op_code) in &ins.addressing_modes {
                if *mode_op_code == op_code {
                    name = Some(ins.name);
                    addr_mode = Some(*mode);
                }
            }
        }

        if let Some(name) = name && let Some(addr_mode) = addr_mode {
            Some(Instruction::new(name, addr_mode, op_code, operand))
        } else {
            None
        }
    }
    
    fn get_all_instructions() -> [InstructionBuilder; NUM_INSTRUCTIONS] {
        [
            // Load/store operations
            InstructionBuilder::new(InstructionName::lda)
                .imm(0xa9)
                .zp(0xa5)
                .zpx(0xb5)
                .abs(0xad)
                .absx(0xbd)
                .absy(0xb9)
                .indx(0xa1)
                .indy(0xb1),
            InstructionBuilder::new(InstructionName::ldx)
                .imm(0xa2)
                .zp(0xa6)
                .zpy(0xb6)
                .abs(0xae)
                .absy(0xbe),
            InstructionBuilder::new(InstructionName::ldy)
                .imm(0xa0)
                .zp(0xa4)
                .zpx(0xb4)
                .abs(0xac)
                .absx(0xbc),
            InstructionBuilder::new(InstructionName::sta)
                .zp(0x85)
                .zpx(0x95)
                .abs(0x8d)
                .absx(0x9d)
                .absy(0x99)
                .indx(0x81)
                .indy(0x91),
            InstructionBuilder::new(InstructionName::stx)
                .zp(0x86)
                .zpy(0x96)
                .abs(0x8e),
            InstructionBuilder::new(InstructionName::sty)
                .zp(0x84)
                .zpx(0x94)
                .abs(0x8c),

            // Register transfers
            InstructionBuilder::new(InstructionName::tax)
                .imp(0xaa),
            InstructionBuilder::new(InstructionName::tay)
                .imp(0xa8),
            InstructionBuilder::new(InstructionName::txa)
                .imp(0x8a),
            InstructionBuilder::new(InstructionName::tya)
                .imp(0x98),

            // Stack operations
            InstructionBuilder::new(InstructionName::tsx)
                .imp(0xba),
            InstructionBuilder::new(InstructionName::txs)
                .imp(0x9a),
            InstructionBuilder::new(InstructionName::pha)
                .imp(0x48),
            InstructionBuilder::new(InstructionName::php)
                .imp(0x08),
            InstructionBuilder::new(InstructionName::pla)
                .imp(0x68),
            InstructionBuilder::new(InstructionName::plp)
                .imp(0x28),

            // Logical
            InstructionBuilder::new(InstructionName::and)
                .imm(0x29)
                .zp(0x25)
                .zpx(0x35)
                .abs(0x2d)
                .absx(0x3d)
                .absy(0x39)
                .indx(0x21)
                .indy(0x31),
            InstructionBuilder::new(InstructionName::eor)
                .imm(0x49)
                .zp(0x45)
                .zpx(0x55)
                .abs(0x4d)
                .absx(0x5d)
                .absy(0x59)
                .indx(0x41)
                .indy(0x51),
            InstructionBuilder::new(InstructionName::ora)
                .imm(0x09)
                .zp(0x05)
                .zpx(0x15)
                .abs(0x0d)
                .absx(0x1d)
                .absy(0x19)
                .indx(0x01)
                .indy(0x11),
            InstructionBuilder::new(InstructionName::bit)
                .zp(0x24)
                .abs(0x2c),

            // Arithmetic
            InstructionBuilder::new(InstructionName::adc)
                .imm(0x69)
                .zp(0x65)
                .zpx(0x75)
                .abs(0x6d)
                .absx(0x7d)
                .absy(0x79)
                .indx(0x61)
                .indy(0x71),
            InstructionBuilder::new(InstructionName::sbc)
                .imm(0xe9)
                .zp(0xe5)
                .zpx(0xf5)
                .abs(0xed)
                .absx(0xfd)
                .absy(0xf9)
                .indx(0xe1)
                .indy(0xf1),
            InstructionBuilder::new(InstructionName::cmp)
                .imm(0xc9)
                .zp(0xc5)
                .zpx(0xd5)
                .abs(0xcd)
                .absx(0xdd)
                .absy(0xd9)
                .indx(0xc1)
                .indy(0xd1),
            InstructionBuilder::new(InstructionName::cpx)
                .imm(0xe0)
                .zp(0xe4)
                .abs(0xec),
            InstructionBuilder::new(InstructionName::cpy)
                .imm(0xc0)
                .zp(0xc4)
                .abs(0xcc),

            // Increments & decrements
            InstructionBuilder::new(InstructionName::inc)
                .zp(0xe6)
                .zpx(0xf6)
                .abs(0xee)
                .absx(0xfe),
            InstructionBuilder::new(InstructionName::inx)
                .imp(0xe8),
            InstructionBuilder::new(InstructionName::iny)
                .imp(0xc8),
            InstructionBuilder::new(InstructionName::dec)
                .zp(0xc6)
                .zpx(0xd6)
                .abs(0xce)
                .absx(0xde),
            InstructionBuilder::new(InstructionName::dex)
                .imp(0xca),
            InstructionBuilder::new(InstructionName::dey)
                .imp(0x88),

            // Shifts
            InstructionBuilder::new(InstructionName::asl)
                .acc(0x0a)
                .zp(0x06)
                .zpx(0x16)
                .abs(0x0e)
                .absx(0x1e),
            InstructionBuilder::new(InstructionName::lsr)
                .acc(0x4a)
                .zp(0x46)
                .zpx(0x56)
                .abs(0x4e)
                .absx(0x5e),
            InstructionBuilder::new(InstructionName::rol)
                .acc(0x2a)
                .zp(0x26)
                .zpx(0x36)
                .abs(0x2e)
                .absx(0x3e),
            InstructionBuilder::new(InstructionName::ror)
                .acc(0x6a)
                .zp(0x66)
                .zpx(0x76)
                .abs(0x6e)
                .absx(0x7e),

            // Jumps & calls
            InstructionBuilder::new(InstructionName::jmp)
                .abs(0x4c)
                .ind(0x6c),
            InstructionBuilder::new(InstructionName::jsr)
                .abs(0x20),
            InstructionBuilder::new(InstructionName::rts)
                .imp(0x60),

            // Branches
            InstructionBuilder::new(InstructionName::bcc)
                .rel(0x90),
            InstructionBuilder::new(InstructionName::bcs)
                .rel(0xb0),
            InstructionBuilder::new(InstructionName::beq)
                .rel(0xf0),
            InstructionBuilder::new(InstructionName::bmi)
                .rel(0x30),
            InstructionBuilder::new(InstructionName::bne)
                .rel(0xd0),
            InstructionBuilder::new(InstructionName::bpl)
                .rel(0x10),
            InstructionBuilder::new(InstructionName::bvc)
                .rel(0x50),
            InstructionBuilder::new(InstructionName::bvs)
                .rel(0x70),

            // Status flag changes
            InstructionBuilder::new(InstructionName::clc)
                .imp(0x18),
            InstructionBuilder::new(InstructionName::cld)
                .imp(0xd8),
            InstructionBuilder::new(InstructionName::cli)
                .imp(0x58),
            InstructionBuilder::new(InstructionName::clv)
                .imp(0xb8),
            InstructionBuilder::new(InstructionName::sec)
                .imp(0x38),
            InstructionBuilder::new(InstructionName::sed)
                .imp(0xf8),
            InstructionBuilder::new(InstructionName::sei)
                .imp(0x78),

            // System functions
            InstructionBuilder::new(InstructionName::brk)
                .imp(0x00),
            InstructionBuilder::new(InstructionName::nop)
                .imp(0xea),
            InstructionBuilder::new(InstructionName::rti)
                .imp(0x40),
        ]
    }
}
