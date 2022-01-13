use super::{compiler_pos::{ExtCompilerPos, CompilerPos}, dif_part_helper::{settings::ErrorSetting, setting_action_result::SettingActionResult}, mnc_checker::ChekerErrorMNC};

pub enum CompilerErrorUnexpEOF{
    NotClosedInclude,
    NotClosedMacro,
    NotClosedUseMacro,
}

pub enum IncludeError{
    MacrosAlreadyDefined{macro_name: String},
    MemInitMergeError{ mmc: usize },
}

pub enum CompilerErrorType{
    UnknownCmd(char),

    FileOpenError,

    UnexpectedEOF, // TODO: {await_close: [CompilerErrorUnexpEOF | char], pos_open: CompilerPos}
    
    EmptyFileName,
    EmptyMacroName,
    BadMacroName(char),
    MacroAlreadyDefined,
    UnknownMacros(String),

    NotAllowedCompileCode,
    NotAllowedCompileMacros,
    NotAllowedCompileSettings,

    NotClosedWhile(Vec<CompilerPos>),
    ClosedWhileWithoutOpen,

    SettingError(ErrorSetting),

    SettingActionError(String, SettingActionResult),
    IncludeError(IncludeError),

    MacroNameCheckError{rule_checker_name: String, error: ChekerErrorMNC},

    Other(String),
}

impl CompilerErrorType{
    pub fn need_pos(&self) -> bool {
        if let Self::FileOpenError = self { false } 
        else { true }
    }
}

pub struct CompilerError{
    pub err_type: CompilerErrorType,
    pub err_stack_pos: Vec<ExtCompilerPos>,
}

macro_rules! new_ce_2p {
    ( $ind:ident, $err_type:expr ) => {
        pub fn $ind(pos: CompilerPos, file_name: String) -> Self { Self::new($err_type, pos, file_name) }
    };
}

macro_rules! new_ce_3p {
    ( $ind:ident, $err_type:expr, $add_param_type:ident ) => {
        pub fn $ind(pos: CompilerPos, file_name: String, x: $add_param_type) -> Self { Self::new($err_type (x), pos, file_name) }
    };
}


type CET = CompilerErrorType;

impl CompilerError{
    pub fn new_file_open_err(file_name: String) -> Self { Self::new_wo_pos(CompilerErrorType::FileOpenError, file_name) }
    new_ce_2p!(new_already_defined, CET::MacroAlreadyDefined);
    new_ce_2p!(new_cant_compile_code, CET::NotAllowedCompileCode);
    new_ce_2p!(new_cant_compile_macros, CET::NotAllowedCompileMacros);
    new_ce_2p!(new_cant_compile_settings, CET::NotAllowedCompileSettings);
    new_ce_2p!(new_empty_name, CET::EmptyFileName);
    new_ce_2p!(new_unexp_eof, CET::UnexpectedEOF);

    new_ce_3p!(new_unknown_macros, CET::UnknownMacros, String);
    new_ce_3p!(new_bad_macro_name, CET::BadMacroName, char);
    new_ce_3p!(new_setting_error, CET::SettingError, ErrorSetting);
    new_ce_3p!(new_include_error, CET::IncludeError, IncludeError);

    pub fn new_setting_action_error(pos: CompilerPos, file_name: String, error: SettingActionResult, in_setting: String) -> Self {
        if error.is_right_rule() { panic!("it is not error!") } 
        Self::new(CET::SettingActionError(in_setting, error), pos, file_name) 
    }

    pub fn new_wo_pos(err_type: CompilerErrorType, file_name: String) -> Self{
        if err_type.need_pos() { panic!("this error need pos!") }
        Self{
            err_type,
            err_stack_pos: vec![ ExtCompilerPos{ pos: None, file_name } ]
        }
    }
    pub fn new(err_type: CompilerErrorType, pos: CompilerPos, file_name: String) -> Self{
        if !err_type.need_pos() { return Self::new_wo_pos(err_type, file_name) }
        Self{
            err_type,
            err_stack_pos: vec![ ExtCompilerPos{ pos: Some(pos), file_name } ]
        }
    }

    pub fn add_err_pos(&mut self, pos: CompilerPos, file_name: String) {
        self.err_stack_pos.push(ExtCompilerPos { pos: Some(pos), file_name })
    }
}