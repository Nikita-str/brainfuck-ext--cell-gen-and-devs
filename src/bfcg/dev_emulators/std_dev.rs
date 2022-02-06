use crate::bfcg::dev_emulators::dev_constructor::{DevCtorErr, DevCtorOk, DevCtor};
use crate::bfcg::dev_emulators::{cem::DevStdCem, com::DevStdCom};
use super::console::{DevConsoleUtf8, DevConsoleNum, DevConsoleAscii};
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
