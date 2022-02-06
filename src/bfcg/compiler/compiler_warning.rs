use crate::bfcg::{vm::port::Port, dev_emulators::dev_name::DevName};

use super::compiler_pos::CompilerPos;


#[derive(Debug)]
pub enum CompilerWarning{
    SettingWarning{ pos: CompilerPos, setting: String, warning: String },
    FromOtherFile{ pos: CompilerPos, warnings: CompilerWarnings},
    /// use it ?only? when merge; when in the same file you know setting string! => use SettingWarning
    OtherPortUsedInOtherFile { pos: CompilerPos, port_name: String, new_port: usize, old_port: usize },
    /// use it ?only? when merge; when in the same file you know setting string! => use SettingWarning
    OtherDevUsedInOtherFile { pos: CompilerPos, port: Port, new_dev_name: DevName, old_dev_name: DevName },
}


#[derive(Debug)]
pub struct CompilerWarnings{
    file_name: Option<String>,
    warnings: Vec<CompilerWarning>,
}

impl CompilerWarnings{
    pub fn new() -> Self {
        Self {
            file_name: None,
            warnings: vec![],
        }
    }

    pub fn is_empty(&self) -> bool { self.warnings.is_empty() }

    pub fn set_file(&mut self, file_name: String) {
        if let Some(x) = &self.file_name { 
            if x == &file_name { return }
            panic!("other file name") 
        } else {
            self.file_name = Some(file_name)
        }
    }

    pub fn add_warning(&mut self, warning: CompilerWarning) { self.warnings.push(warning) }
}