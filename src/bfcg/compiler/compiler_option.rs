use std::marker::PhantomData;

use super::{comand_compiler::CmdCompiler, dif_part_helper::setting_action::SettingActions, mnc_checker::HolderChekerMNC, compiler_error::CompilerErrorType};


#[derive(Clone, Copy)]
pub enum CanCompile{
    OnlyMacros,
    OnlySettings,
    MacroAndSettings,
    All, // code + settings + macros
}

impl CanCompile{
    pub fn can_compile_code(&self) -> bool {
        if let CanCompile::All = self { true } else { false }
    }

    pub fn can_compile_macro(&self) -> bool {
        match self {
            CanCompile::OnlyMacros | CanCompile::MacroAndSettings | CanCompile::All => true,
            _ => false,
        }
    }
    
    pub fn can_compile_settings(&self) -> bool {
        match self {
            CanCompile::OnlySettings | CanCompile::MacroAndSettings | CanCompile::All => true,
            _ => false,
        }
    }    
}

#[derive(Debug, Clone, Copy)]
pub enum MemInitType{
    Dirrect,
    BeforeCode,
    AfterCodeByConcat,
}

impl MemInitType{
    pub fn mem_init_only_before_code(&self) -> bool { if let Self::BeforeCode = self { true } else { false } }
}

pub struct CompilerOption<'a, CC, T>
where CC: CmdCompiler<T>,
{
    pub phantom: PhantomData<T>,
    pub can_compile: CanCompile,
    pub mem_init_type: MemInitType,
    pub cmd_compiler: Option<CC>, // TODO: &'a mut CC
    pub setting_action: &'a SettingActions<T>,
    pub default_settings: Vec<String>, 
    pub mnc_checker: &'a HolderChekerMNC<'a>,
}

impl<'a, CC, T> CompilerOption<'a, CC, T>
where CC: CmdCompiler<T>
{
    pub fn new(
        mem_init_type: MemInitType, 
        cmd_compiler: CC, 
        setting_action: &'a SettingActions<T>, 
        default_settings: Vec<String>,
        mnc_checker: &'a HolderChekerMNC<'a>) -> Self 
    {
        Self {
            phantom: PhantomData,
            can_compile: CanCompile::All, 
            mem_init_type, 
            cmd_compiler: Some(cmd_compiler), 
            setting_action, 
            default_settings, 
            mnc_checker
        }
    }

    /// # panic
    /// * if can compile code (can_compile == All)
    pub fn new_only(&self, can_compile: CanCompile) -> Self{
        if let CanCompile::All = can_compile { panic!("when you use other files you dont whant to compile code") }
        Self {
            phantom: PhantomData,
            can_compile,
            mem_init_type: MemInitType::BeforeCode,
            cmd_compiler: None,
            setting_action: self.setting_action,
            default_settings: vec![], // default settings must be processed only in first file
            mnc_checker: self.mnc_checker,
        }
    }

    pub fn need_processed_default_settings(&self) -> bool { !self.default_settings.is_empty() }
    pub fn get_default_settings(&mut self) -> Vec<String> { std::mem::take(&mut self.default_settings) }

    pub fn can_compile_code(&self) -> bool { self.can_compile.can_compile_code() }
    pub fn can_compile_macro(&self) -> bool { self.can_compile.can_compile_macro() }
    pub fn can_compile_settings(&self) -> bool { self.can_compile.can_compile_settings() }

    pub fn mnc_check(&self, macro_name: &str, macro_code: &str) -> Option<CompilerErrorType> {
        if let Some((name, error)) = self.mnc_checker.check_all(macro_name, macro_code){
            return Some( CompilerErrorType::MacroNameCheckError{ rule_checker_name: name, error } )
        }
        None
    }
}