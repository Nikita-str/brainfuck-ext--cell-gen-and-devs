use super::super::{compiler_pos::CompilerPos, compiler_error::{CompilerErrorType}};

pub trait CmdCompiler<T>{
    // cause mut self => real cmd may look like "xyz" | "i" | "use" | "arch" | "btw"
    fn cmd_compile(&mut self, cmd: char, pos: CompilerPos) -> Option<CompilerErrorType>; 

    fn get_program(self) -> Result<Vec<T>, CompilerErrorType>;
}