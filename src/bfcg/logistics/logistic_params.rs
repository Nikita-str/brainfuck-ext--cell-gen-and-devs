use clap::Parser;

use crate::HardwareInfo;

const STD_GEN_PATH: &'static str = "generated/";

/// Program that contain utilities for work with bf-ext
/// such as: compiler, virtual machine, disasembling(only for std compiler) 
#[derive(Parser, Debug)]
#[clap(about, long_about = None)]
pub struct LogisticParams {

    /// bf-ext file
    #[clap(short, long)] 
    pub file: String,

    /// do you want to generate a disasm?
    #[clap(long)] 
    pub need_disasm: bool,

    /// do you want to generate a binary?
    #[clap(long)] 
    pub need_bin: bool,

    /// path to folder where will be created disasm
    /// 
    #[clap(long, default_value = STD_GEN_PATH)] 
    pub path_disasm: String,

    /// path to folder where will be created binary file
    #[clap(long, default_value = STD_GEN_PATH)] 
    pub path_bin: String,

    /// \[HARDWARE INFO\]: port amount
    #[clap(short, long, default_value_t = 64)] 
    pub hardware_port_amount: usize,
    
    /// \[HARDWARE INFO\]: max jump size
    #[clap(long, default_value_t = 1 << 20)] 
    pub hardware_max_jump_size: usize,
    
    /// \[HARDWARE INFO\]: COM port
    #[clap(long, default_value_t = 1)] 
    pub hardware_com_port: usize,

    /// \[HARDWARE INFO\]: CEM port
    #[clap(long, default_value_t = 2)] 
    pub hardware_cem_port: usize,

    // TODO: additional device: ?Subcomand? that take Vec<String> 
    
    // TODO: MNC checker

    // TODO: MemInitType::
}

impl LogisticParams {
    pub fn new_empty(file_path: &str) -> Self {
        Self {
            file: String::from(file_path),

            need_disasm: false,
            need_bin: false,

            path_disasm: String::from(STD_GEN_PATH),
            path_bin: String::from(STD_GEN_PATH),

            hardware_com_port: 1, 
            hardware_cem_port: 2,
            hardware_max_jump_size: 1 << 20,
            hardware_port_amount: 64,
        }
    }

    pub fn on_disasm(&mut self, disasm_path: Option<&str>) {
        self.need_disasm = true;
        if let Some(x) = disasm_path { self.path_disasm = String::from(x) }
    }
    
    pub fn on_bin(&mut self, bin_path: Option<&str>) {
        self.need_bin = true;
        if let Some(x) = bin_path { self.path_bin = String::from(x) }
    }

    pub fn off_disasm(&mut self) { self.need_disasm = false }
    pub fn off_bin(&mut self) { self.need_bin = false }

    pub fn change_hardware(&mut self, hw_info: &HardwareInfo) {
        self.hardware_port_amount = hw_info.max_port_amount;
        self.hardware_max_jump_size = hw_info.max_jump_size;
        self.hardware_com_port = hw_info.default_com_port;
        self.hardware_cem_port = hw_info.default_cem_port;
    }
}