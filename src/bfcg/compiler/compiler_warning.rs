use crate::bfcg::{vm::port::Port, dev_emulators::dev_name::DevName, general::PAD};

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

impl ToString for CompilerWarnings {
    fn to_string(&self) -> String {
        let mut ret = String::new();
        ret += &format!("warning in <{}>:", if let Some(x) = &self.file_name { x } else { "unknown file" });
        for x in &self.warnings {
            ret += PAD;
            ret += &x.to_string().replace("\n", &format!("\n{}{}", PAD, PAD));
            ret += "\n";
        }
        ret
    }
}

impl ToString for CompilerWarning {
    fn to_string(&self) -> String {
        match self {
            Self::FromOtherFile{pos, warnings} => { format!("at position {}  {}", pos.to_string(), warnings.to_string()) }
            Self::SettingWarning{pos, setting, warning} => { 
                format!("in setting {} at positon {}: {}", pos.to_string(), setting, warning) 
            }
            Self::OtherDevUsedInOtherFile{pos, port, new_dev_name, old_dev_name} => {
                format!(
                    "other dev used in port {} at position {}:  {} --> {}", 
                    port.to_string(), pos.to_string(), old_dev_name.to_string(), new_dev_name.to_string()
                ) 
            }
            Self::OtherPortUsedInOtherFile{pos, port_name, old_port, new_port} => {
                format!("other port number used for port-name {} at position {}:  {} --> {}", port_name, pos.to_string(), old_port, new_port) 
            }
        }
    }
}