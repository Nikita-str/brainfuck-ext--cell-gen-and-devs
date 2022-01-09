use super::{valid_cmd::ValidCMD, compiler_pos::CompilerPos, compiler_error::CompilerError};

pub trait CmdCompiler<T>{
    fn cmd_compile(&mut self, cmd: char, pos: CompilerPos) -> Option<CompilerError>; // cause mut self => real cmd may look like "xyz" | "i" | "use" | "arch" | "btw"

    fn get_program(self) -> Result<Vec<T>, CompilerError>;
}



pub struct InterpreterCmdCompiler{
    program: Vec<ValidCMD>,

    open_while: Vec<()>,
}

impl InterpreterCmdCompiler{
    pub fn new() -> Self{
        Self { 
            program: vec![],
            open_while: vec![], 
        }
    }
}

impl CmdCompiler<ValidCMD> for InterpreterCmdCompiler{
    fn cmd_compile(&mut self, cmd: char, pos: CompilerPos) -> Option<CompilerError> {
        todo!("----")
    }

    fn get_program(self) -> Result<Vec<ValidCMD>, CompilerError> { 
        if self.open_while.len() == 0 { Ok(self.program) }
        else { todo!("Err") } 
    }
}
