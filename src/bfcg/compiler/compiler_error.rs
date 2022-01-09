use super::compiler_pos::{ExtCompilerPos, CompilerPos};

pub enum CompilerErrorUnexpEOF{
    NotClosedInclude,
    NotClosedMacro,
    NotClosedUseMacro,
}

pub enum CompilerErrorType{
    FileOpenError,

    UnexpectedEOF,
    
    EmptyFileName,
    EmptyMacroName,
    BadMacroName(char),
    MacroAlreadyDefined,
    CodeInMacros,
    UnknownMacros(String),

    NotClosedWhile,
    ClosedWhileWithoutOpen,
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

type CET = CompilerErrorType;

impl CompilerError{
    pub fn new_file_open_err(file_name: String) -> Self { Self::new_wo_pos(CompilerErrorType::FileOpenError, file_name) }
    new_ce_2p!(new_already_defined, CET::MacroAlreadyDefined);
    new_ce_2p!(new_code_in_macros, CET::CodeInMacros);
    new_ce_2p!(new_empty_name, CET::EmptyFileName);
    new_ce_2p!(new_unexp_eof, CET::UnexpectedEOF);
    pub fn new_unknown_macros(pos: CompilerPos, file_name: String, macros_name: String) -> Self { 
        Self::new(CET::UnknownMacros(macros_name), pos, file_name) 
    }
    pub fn new_bad_macro_name(pos: CompilerPos, file_name: String, bad_char: char) -> Self { 
        Self::new(CET::BadMacroName(bad_char), pos, file_name) 
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