use crate::bfcg::{compiler::{valid_cmd::ValidCMD, compiler_pos::CompilerPos, compiler_error::CompilerErrorType}, general::TryIntoMutRef};

use super::{cmd_compiler::CmdCompiler, program_concat::{ProgramConcat, self}, NullPortNameHandler};


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

impl ProgramConcat<ValidCMD> for InterpreterCmdCompiler{
    fn program_concat(p1: Vec<ValidCMD>, p2: Vec<ValidCMD>) -> Vec<ValidCMD> {
        program_concat::default_program_concat(p1, p2)
    }
}

impl TryIntoMutRef<NullPortNameHandler> for InterpreterCmdCompiler{
    fn try_into_mut_ref(&mut self) -> Option<&mut NullPortNameHandler> { None }
}