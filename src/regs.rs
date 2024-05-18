use bitflags::bitflags;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CpuFlags(u8);

bitflags! {
    impl CpuFlags: u8 {
        const NONE          = 0b0000_0000;
        const CARRY         = 0b0000_0001;
        const ZERO          = 0b0000_0010;
        const INT_DISABLE   = 0b0000_0100;
        const DEC_MODE      = 0b0000_1000;
        const BREAK         = 0b0001_0000;
        const OVERFLOW      = 0b0100_0000;
        const NEG           = 0b1000_0000;
    }
}

impl Display for CpuFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Regs {
    pub pc: u16,
    pub sp: u8,

    pub a: u8,
    pub x: u8,
    pub y: u8,

    pub flags: CpuFlags,
}

impl Regs {
    pub const fn new() -> Self {
        Self {
            pc: 0,
            sp: 0xff,
            a: 0,
            x: 0,
            y: 0,
            flags: CpuFlags::NONE,
        }
    }
}

impl Display for Regs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Regs(pc={},sp={},a={},x={},y={},flags={})",
            self.pc, self.sp, self.a, self.x, self.y, self.flags
        )
    }
}
