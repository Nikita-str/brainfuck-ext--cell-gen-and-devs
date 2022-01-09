use super::compiler_pos::CompilerPos;

pub enum CompilerErrorType{
    UnexpectedEOF,
    EmptyFileName,
    EmptyMacroName,
    NotClosedWhile,
    ClosedWhileWithoutOpen,
}

pub struct CompilerError<'a>{
    pub err_type: CompilerErrorType,
    pub pos: Vec<CompilerPos<'a>>,
}