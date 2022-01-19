use std::collections::LinkedList;
use crate::bfcg::{general::{se_fn::std_se_encoding, self}, compiler::{compiler_pos::CompilerPos, compiler_error::CompilerErrorType, code_gen, valid_cmd::ValidCMD}};

use super::CmdCompiler;

pub enum StdCmdNames{
    Pass, // 0x00
    Test, // 0x01
    Cur(usize), // 0x02:SE
    Set, // 0x03 + CEM: SE
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
            Self::Set => Self::one_ll(0x03).into_iter(),
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

    /// amount of byte that max need for compile cmd_seq
    /// ## panic
    /// if it cant be compiled
    fn reserve_cmd_seq(&self, cmd_seq: &str) -> usize{
        let mut len = 0; // amount of byte that max need for setting cell value 
        for cmd in cmd_seq.chars() {
            if let Ok(x) = self.cmd_compile_to_byte(cmd, CompilerPos::new()) {
                len += x.len();
            } else {
                panic!("cant compile cmd_seq (bad char: {}) :/", cmd)
            }
        }        
        len
    }

    /// amount of byte that max need for compile one_cmd
    /// ## panic
    /// if it cant be compiled
    fn reserve_cmd(&self, one_cmd: ValidCMD) -> usize{
        let cmd = one_cmd.std_to_char();
        if let Ok(x) = self.cmd_compile_to_byte(cmd, CompilerPos::new()) {
            return x.len()
        } else {
            panic!("cant compile cmd_seq (bad char: {}) :/", cmd)
        }
    }

    /// ### Ru comment version:
    /// вычисляем размер (в байтах) который нужно оставить 
    /// для генерации в CEM SE последовательности размера 
    /// числа не больше max_x 
    fn reserve_prog_space_cem_se_gen(&self, max_x: usize) -> usize{
        // in the worst case if max is 0 we must nullify:
        let nullify_len = self.reserve_cmd(ValidCMD::ZeroedCell);
        
        let se_byte_len = general::se_fn::std_se_encoding(max_x).len();
        
        // amount of byte that max need for setting cell value 
        let one_byte_set_len = self.reserve_cmd_seq(&code_gen::cgen_set_cell_max_cmds()); 

        // amount of byte that max need for `>`
        // # spoiler: it always one byte in cur realization
        let cell_shift_len = self.reserve_cmd(ValidCMD::NextCell);

        // baddest case: 0 set_cell_value(LONGEST_FOR_CODING_BYTE) > ... set_cell_value(LONGEST_FOR_CODING_BYTE)
        let len_of_se = nullify_len + se_byte_len * one_byte_set_len + (se_byte_len - 1) * cell_shift_len;

        len_of_se
    }

    /// for initing pr need: 
    /// + create in CEM: 0 X\[1\] ... X\[N\] where X\[1\]...X\[N\] is SE for W_PR (where W is CONSOLE/WIN/???)
    /// + set CEM ptr on X\[1\]   |  `[<]>`
    /// + CUR\[X_PR\] SET
    /// + set CEM ptr on X\[N\] | `[>]<`
    /// + clear all X\[i\] and set CEM ptr on 0 | `[0<]`
    fn reserve_one_pr_init(&self) -> usize {
        let first_sh = self.reserve_cmd(ValidCMD::NextCell);
        let se_gen = self.reserve_prog_space_cem_se_gen(self.max_port_amount - 1);
        let to_start = self.reserve_cmd_seq(&code_gen::cgen_move_to_next_after_left_zero());
        let cur_and_set = StdCmdNames::Cur(MAX_PR).to_u8_seq().count() + StdCmdNames::Set.to_u8_seq().count();
        let to_end = self.reserve_cmd_seq(&code_gen::cgen_move_to_prev_before_right_zero());
        let clear = self.reserve_cmd_seq(&code_gen::cgen_zero_while_not_zero());

        let byte_len = first_sh + se_gen + to_start + cur_and_set + to_end + clear;
        byte_len
    }

    fn reserve_initial_program_space(&self) -> Vec<u8>{
        // CUR[X_PR] SET[Z] where Z < max_port_amount
        let one_pr_sz = self.reserve_one_pr_init();
        let mut initial_pass = vec![];
        for _ in 0..(one_pr_sz * (MAX_PR - 1)) { // MAX_PR - 1 cause USER_PR not need to set
            for x in StdCmdNames::Pass.to_u8_seq() { initial_pass.push(x) }
        }
        initial_pass
    }

    fn program_init(&self) -> Vec<u8> {
        let mut ret = self.reserve_initial_program_space();
        for x in StdCmdNames::Cur(0).to_u8_seq() { ret.push(x); }
        for x in StdCmdNames::Set.to_u8_seq() { ret.push(x); }
        ret
    }

    /// ## params
    /// + max_jump_size: if you dont know => use memory size
    pub fn new(max_port_amount: usize, max_jump_size: usize) -> Self{
        if max_port_amount < MIN_PORT_AMOUNT { panic!("no enough port for all std devs (need minimum ports for console & win)") }
        let mut ret = Self{ 
            program: vec![],//Self::program_init(max_port_amount),
            open_while: vec![],
            cur_port_reg: 0,
            max_port_amount,
            max_jump_size,
            reserved_use: false,
        };
        ret.program = ret.program_init();
        ret
    }

    fn cmd_compile_to_byte(&self, cmd: char, pos: CompilerPos) -> Result<Vec<u8>, CompilerErrorType>{
        match cmd {

            _ => return Err(CompilerErrorType::UnknownCmd(cmd)),
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