use glium::texture::InternalFormatType;

use super::{valid_cmd::ValidCMD, compiler_pos::CompilerPos};

pub trait CmdCompiler<T>{
    fn cmd_compile(&mut self, cmd: char, pos: CompilerPos); // cause mut self => real cmd may look like "xyz" | "i" | "use" | "arch" | "btw"

    fn can_program_ended(&self) -> bool;
    fn get_program(self) -> Vec<T>;
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
    fn cmd_compile(&mut self, cmd: char, pos: CompilerPos) {
        todo!("----")
    }

    fn can_program_ended(&self) -> bool { self.open_while.len() == 0 }
    fn get_program(self) -> Vec<ValidCMD> { self.program }
}
