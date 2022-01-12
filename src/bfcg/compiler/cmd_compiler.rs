use super::{valid_cmd::ValidCMD, compiler_pos::CompilerPos, compiler_error::{CompilerErrorType}};

pub trait CmdCompiler<T>{
    // cause mut self => real cmd may look like "xyz" | "i" | "use" | "arch" | "btw"
    fn cmd_compile(&mut self, cmd: char, pos: CompilerPos) -> Option<CompilerErrorType>; 

    fn get_program(self) -> Result<Vec<T>, CompilerErrorType>;
}


pub struct InterpreterCmdCompiler{
    program: Vec<ValidCMD>,
    open_while: Vec<CompilerPos>,
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
    fn cmd_compile(&mut self, cmd: char, pos: CompilerPos) -> Option<CompilerErrorType> {
        if let Some(cmd) = ValidCMD::std_parse_char(cmd) {
            if cmd.is_start_while() { self.open_while.push(pos) }
            if cmd.is_end_while() { 
                if let None = self.open_while.pop() {
                    return Some(CompilerErrorType::ClosedWhileWithoutOpen)
                } 
            }
            self.program.push(cmd);
            None
        } else { Some(CompilerErrorType::UnknownCmd(cmd)) }
    }

    fn get_program(self) -> Result<Vec<ValidCMD>, CompilerErrorType> { 
        if self.open_while.len() == 0 { Ok(self.program) }
        else { Err(CompilerErrorType::NotClosedWhile(self.open_while)) } 
    }
}
