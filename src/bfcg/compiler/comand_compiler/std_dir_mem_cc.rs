use std::collections::LinkedList;
use crate::bfcg::{general::se_fn::std_se_encoding, compiler::{compiler_pos::CompilerPos, compiler_error::CompilerErrorType}};

use super::CmdCompiler;

pub enum StdCmdNames{
    Pass, // 0x00
    Test, // 0x01
    Cur(usize), // 0x02:SE
    Set(usize), // 0x03:SE
    Read, // 0x04
    Write, // 0x05
}



impl StdCmdNames{
    fn one_ll(x: u8) -> LinkedList<u8> { 
        let mut ret = LinkedList::new();
        ret.push_back(x);
        ret
     }

     fn add_front_ll(mut ll: LinkedList<u8>, front_value: u8) -> LinkedList<u8>{
        ll.push_front(front_value);
        ll
     }

    pub fn to_u8_seq(&self) -> impl Iterator<Item = u8> {
        match self {
            Self::Pass => Self::one_ll(0x00).into_iter(),
            Self::Test => Self::one_ll(0x01).into_iter(),
            Self::Cur(se) => Self::add_front_ll(std_se_encoding(*se), 0x02).into_iter(),
            Self::Set(se) => Self::add_front_ll(std_se_encoding(*se), 0x03).into_iter(),
            Self::Read => Self::one_ll(0x04).into_iter(),
            Self::Write => Self::one_ll(0x05).into_iter(),
        }
    }
}

/// PR is PORT REG
const USER_PR: usize = 0;
const CONSOLE_PR: usize = USER_PR + 1;
const WIN_PR: usize = CONSOLE_PR + 1;
const MAX_PR: usize = WIN_PR + 1;

pub const MIN_PORT_AMOUNT: usize = 2;

/// standart cc with cpu direct memory access (Inc, Dec, NextCell, JumpRight, ...)
pub struct StdDirMemCmdCompiler{
    program: Vec<u8>,
    open_while: Vec<CompilerPos>,
    /// changed after CUR:SE on SE, this need for
    ///   
    /// ```.,..@,.rrw```
    /// 
    /// compiled into
    /// 
    /// ``` CUR[CONSOLE_PR] SET[console_port] WR READ WR WR CUR[WIN_PR] SET[win_port] CUR[CONSOLE_PR] READ WR CUR[USER_PR] READ READ WR```
    cur_port_reg: usize,
    max_port_amount: usize,
    max_jump_size: usize,
    reserved_use: bool,
}

impl StdDirMemCmdCompiler{
    fn reserve_initial_program_space(max_port_amount: usize) -> Vec<u8>{
        // CUR[X_PR] SET[Z] where Z < max_port_amount
        let one_pr_sz = StdCmdNames::Cur(MAX_PR).to_u8_seq().count() + StdCmdNames::Set(max_port_amount - 1).to_u8_seq().count();
        let mut initial_pass = vec![];
        for _ in 0..(one_pr_sz * (MAX_PR - 1)) { // MAX_PR - 1 cause USER_PR not need to set
            for x in StdCmdNames::Pass.to_u8_seq() { initial_pass.push(x) }
        }
        initial_pass
    }

    fn program_init(max_port_amount: usize) -> Vec<u8>{
        let mut ret = Self::reserve_initial_program_space(max_port_amount);
        for x in StdCmdNames::Cur(0).to_u8_seq() { ret.push(x); }
        for x in StdCmdNames::Set(0).to_u8_seq() { ret.push(x); }
        ret
    }

    /// ## params
    /// + max_jump_size: if you dont know => use memory size
    pub fn new(max_port_amount: usize, max_jump_size: usize) -> Self{
        if max_port_amount < MIN_PORT_AMOUNT { panic!("no enough port for all std devs (need minimum ports for console & win)") }
        Self{ 
            program: Self::program_init(max_port_amount),
            open_while: vec![],
            cur_port_reg: 0,
            max_port_amount,
            max_jump_size,
            reserved_use: false,
        }
    }
}

impl CmdCompiler<u8> for StdDirMemCmdCompiler{
    fn cmd_compile(&mut self, cmd: char, pos: CompilerPos) -> Option<CompilerErrorType> {
        match cmd {
            _ => return Some(CompilerErrorType::UnknownCmd(cmd)),
        }
    }

    fn get_program(self) -> Result<Vec<u8>, CompilerErrorType> {
        if !self.open_while.is_empty() {
            Err(CompilerErrorType::NotClosedWhile(self.open_while))
        } else {
            Ok(self.program)
        }
    }
}