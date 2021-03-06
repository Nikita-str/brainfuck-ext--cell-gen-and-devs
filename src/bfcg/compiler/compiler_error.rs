use crate::bfcg::general::{PAD, HALF_PAD};

use super::{compiler_pos::{ExtCompilerPos, CompilerPos}, dif_part_helper::{settings::ErrorSetting, setting_action_result::SettingActionResult}, mnc_checker::ChekerErrorMNC, compiler_info::MacroCodeProcessError};

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
    MacroAlreadyDefined(String),
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

    /// if None => EOF; if Some(c) => any char except MACRO_USE_CHAR
    ExpectedSpaceSeparatedSeq(Option<char>),

    MacroCodeProcessError(MacroCodeProcessError),

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
    ( $ind:ident, $err_type:expr, $add_param_type:ty ) => {
        pub fn $ind(pos: CompilerPos, file_name: String, x: $add_param_type) -> Self { Self::new($err_type (x), pos, file_name) }
    };
}

type CET = CompilerErrorType;

impl CompilerError{
    pub fn new_file_open_err(file_name: String) -> Self { Self::new_wo_pos(CompilerErrorType::FileOpenError, file_name) }
    new_ce_3p!(new_already_defined, CET::MacroAlreadyDefined, String);
    new_ce_2p!(new_cant_compile_code, CET::NotAllowedCompileCode);
    new_ce_2p!(new_cant_compile_macros, CET::NotAllowedCompileMacros);
    new_ce_2p!(new_cant_compile_settings, CET::NotAllowedCompileSettings);
    new_ce_2p!(new_empty_name, CET::EmptyFileName);
    new_ce_2p!(new_unexp_eof, CET::UnexpectedEOF);

    new_ce_3p!(new_unknown_macros, CET::UnknownMacros, String);
    new_ce_3p!(new_bad_macro_name, CET::BadMacroName, char);
    new_ce_3p!(new_setting_error, CET::SettingError, ErrorSetting);
    new_ce_3p!(new_include_error, CET::IncludeError, IncludeError);
    new_ce_3p!(new_expect_space_sep, CET::ExpectedSpaceSeparatedSeq, Option<char>);

    new_ce_3p!(new_macro_code_process_error, CET::MacroCodeProcessError, MacroCodeProcessError);

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


// -------------------------------------------------------
// [+] TO STRING:

impl ToString for CompilerError {
    fn to_string(&self) -> String {
        let mut shift = String::new();

        let mut ret = 
        if self.err_stack_pos.is_empty() { format!("<in unknown file at unknown position>:\n") }
        else {
            let mut temp = String::new();
            for x in &self.err_stack_pos {
                temp += &x.to_string();
                temp += &format!(":\n{}", shift);
                shift += HALF_PAD;
            }
            temp
        };

        ret += &format!("{}error:\n", shift);
        shift += PAD;

        ret += PAD;
        ret += &self.err_type.to_string().replace(&format!("\n{}", PAD), &shift);

        ret
    }
}

impl ToString for CompilerErrorType {
    fn to_string(&self) -> String {
        match self {
            Self::UnknownCmd(cmd) => format!("unknown cmd: `{}`", cmd),

            Self::UnexpectedEOF => String::from("unexpected end of file (EOF)"),
            Self::FileOpenError => String::from("error during open file"),
            Self::EmptyFileName => String::from("empty file name in include"),

            Self::EmptyMacroName => String::from("empty macro name"),
            Self::BadMacroName(char) => format!("bad macro name, use of unallowed character ('{}')", char),
            Self::MacroAlreadyDefined(name) => format!("macro with name \"{name}\" already defined"),
            Self::UnknownMacros(x) => format!("use of unknown macro (\"{}\")", x),

            Self::NotAllowedCompileCode => format!("this file included without allowing to compile code"),
            Self::NotAllowedCompileMacros => format!("this file included without allowing to compile macros"),
            Self::NotAllowedCompileSettings => format!("this file included without allowing to compile settings"),

            Self::ClosedWhileWithoutOpen => format!("close while without open"),
            Self::NotClosedWhile(vec_of_open) => {
                let mut ret = String::from("not closed while:");
                assert!(!vec_of_open.is_empty());
                for x in vec_of_open {
                    ret.push('\n');
                    ret += PAD;
                    ret += &x.to_string();
                }
                ret
            }

            Self::IncludeError(x) => format!("include error: {}", x.to_string()),
            Self::SettingError(x) => format!("in setting: {}", x.to_string()),
            Self::MacroNameCheckError{rule_checker_name, error} => {
                format!("in rule checker <{}> error: {}", rule_checker_name, error.to_string())
            }
            Self::SettingActionError(setting, err) => {
                let mut ret = format!("in setting \"{}\":", setting);
                ret += &err.result_type.to_string().replace("\n", &("\n".to_owned() + PAD));
                ret
            }

            Self::ExpectedSpaceSeparatedSeq(c) => {
                match *c {
                    None => format!("unexpected end of file (EOF): expected space-separated-macro-use sequence (%%%)"),
                    Some(c) => {
                        let c = if c.is_whitespace() { ' ' } else { c };
                        format!("unexpected char '{c}': expected space-separated-macro-use sequence (%%%) but instead was \"%{c}\" or \"%%{c}\"")
                    }
                }
            }

            Self::MacroCodeProcessError(err) => format!("error during macro code processing: {}", err.to_string()),

            Self::Other(x) => String::from(x),
        }
    }
}

impl ToString for IncludeError {
    fn to_string(&self) -> String {
        match self {
            Self::MacrosAlreadyDefined { macro_name } => format!("macro with name \"{}\" already defined", macro_name),
            Self::MemInitMergeError { mmc } => format!("memory can not to merge: cell after main mem cell {} initing twice", mmc),
        }
    }
}

// [-] TO STRING:
// -------------------------------------------------------
