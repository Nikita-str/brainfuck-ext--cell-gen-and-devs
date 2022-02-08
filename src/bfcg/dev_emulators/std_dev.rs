use crate::bfcg::dev_emulators::dev_constructor::{DevCtorErr, DevCtorOk, DevCtor};
use crate::bfcg::dev_emulators::{cem::DevStdCem, com::DevStdCom};
use super::console::{DevConsoleUtf8, DevConsoleNum, DevConsoleAscii};
use super::dev_constructor::SpecialWinCtor;
use super::win::win::SpecialWin;
use super::{dev_name::DevName};


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum StdDevName{
    Console,
    Win,
    CellMem,
    CmdMem,
}



pub fn std_dev_constructor(dev_name: &DevName) -> Result<DevCtorOk, DevCtorErr> {
    match dev_name.get_name() {
        "CEM" | "cem" | "std-cem" | "STD-CEM" => { DevStdCem::dev_ctor(dev_name.get_params()) }
        "COM" | "com" | "std-com" | "STD-COM" => { DevStdCom::dev_ctor(dev_name.get_params()) }
        "utf8-console" => { DevConsoleUtf8::dev_ctor(dev_name.get_params()) }
        "ascii-console" => { DevConsoleAscii::dev_ctor(dev_name.get_params()) }
        "num-console" => { DevConsoleNum::dev_ctor(dev_name.get_params()) }

        _ => { Err(DevCtorErr::UnknownName(dev_name.get_name().to_owned())) }
    }
}

pub fn get_std_win_dev_type(dev_name: &DevName) -> Option<StdWinDev> {
    match dev_name.get_name() {
        "std-win" => Some(StdWinDev::Screen),
        _ => None,
    }
}

pub fn std_win_spec_constructor(dev_name: &DevName) -> Result<SpecialWin, DevCtorErr> {
    match dev_name.get_name() {
        "std-win" => { SpecialWin::special_win_ctor(dev_name.get_params()) }
        _ => { Err(DevCtorErr::UnknownName(dev_name.get_name().to_owned())) }
    }
}

/// TODO: we can add key press, mouse and so on:
pub enum StdWinDev{
    Screen,
} 

impl StdWinDev {
    pub const fn is_screen(&self) -> bool { matches!(self, Self::Screen) }
}