use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
    str::FromStr,
};

use clap::Parser;

use crate::regs::{CpuFlags, Regs};

#[derive(Parser)]
pub struct Args {
    pub path: PathBuf,
    #[arg(long, default_value = None)]
    pub memory: Option<PathBuf>,
    /// How to use: --regs x=3,y=1
    #[arg(long)]
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