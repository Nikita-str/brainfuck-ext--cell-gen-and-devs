use super::compiler_pos::CompilerPos;


#[derive(Debug)]
pub enum CompilerWarning{
    SettingWarning{ pos: CompilerPos, setting: String, warning: String }
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