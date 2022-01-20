use std::collections::LinkedList;
use crate::bfcg::{general::{se_fn::std_se_encoding, self}, compiler::{compiler_pos::CompilerPos, compiler_error::CompilerErrorType, code_gen, valid_cmd::ValidCMD}};
use super::{CmdCompiler, PortNameHandler, sdm_cc_additional_info::{SDMCCAditionalInfo, PrPrepared}};

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
pub(in super) const MAX_PR: usize = WIN_PR + 1;

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

    inner_info: SDMCCAditionalInfo,
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
        let one_pr_sz = 
            if self.inner_info.get_pr_reserve_sz() == 0 { self.reserve_one_pr_init() } 
            else { self.inner_info.get_pr_reserve_sz() };
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

    fn cgen_set_port(&mut self, pr: &PrPrepared, port_num: usize) {
        let pr_index = pr.to_index();
        let set_byte = move |x: &mut Self, cgen_compiled_sz: &mut usize, byte|{
            x.program[pr_index * x.inner_info.get_pr_reserve_sz() + *cgen_compiled_sz] = byte;
            *cgen_compiled_sz += 1;
        };
        let cgen_compile = |cgen: &str, x: &mut Self, cgen_compiled_sz: &mut usize|{
            for cmd in cgen.chars() {
                if let Ok(bytes) = x.cmd_compile_to_byte(cmd, CompilerPos::new()) {
                    for byte in bytes { set_byte(x, cgen_compiled_sz, byte); } 
                } else {
                    panic!("cant compile auto gen code")
                }
            }
        };


        if self.inner_info.get_pr_reserve_sz() == 0 { panic!("uninit reserved size") }
        let mut cgen = String::new();
        let mut cgen_compiled_sz = 0;

        // #############################################
        // CGEN[START]
        cgen.push(ValidCMD::NextCell.std_to_char());
        
        let cell_values = general::se_fn::std_se_encoding(port_num);
        code_gen::add_cgen_init_se_cem(&mut cgen, cell_values, false);

        code_gen::add_cgen_move_to_next_after_left_zero(&mut cgen);

        cgen_compile(&cgen, self, &mut cgen_compiled_sz);

        // #############################################
        // CGEN[CENTER]    (CUR + SET)
        let cur = if let PrPrepared::Console = pr { CONSOLE_PR } else { WIN_PR };
        for byte in StdCmdNames::Cur(cur).to_u8_seq() { set_byte(self, &mut cgen_compiled_sz, byte); }
        for byte in StdCmdNames::Set.to_u8_seq() { set_byte(self, &mut cgen_compiled_sz, byte); }

        // #############################################
        // CGEN[END]   
        cgen.clear();
        code_gen::add_cgen_move_to_prev_before_right_zero(&mut cgen);
        code_gen::add_cgen_zero_while_not_zero(&mut cgen);
        cgen_compile(&cgen, self, &mut cgen_compiled_sz);

        if cgen_compiled_sz > self.inner_info.get_pr_reserve_sz() { panic!("[ALGO ERROR]: wrong reserved size counted") }

        // nullify if already use this cell:
        for _ in 0..(self.inner_info.get_pr_reserve_sz() - cgen_compiled_sz) {
            set_byte(self, &mut cgen_compiled_sz, 0x00);
        }

        if cgen_compiled_sz != self.inner_info.get_pr_reserve_sz() { panic!("[ALGO ERROR] :/") }
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

            inner_info: SDMCCAditionalInfo::new(),
        };
        ret.inner_info.set_pr_reserve_sz(ret.reserve_one_pr_init());
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

impl PortNameHandler for StdDirMemCmdCompiler{
    fn need_port_name_handle(&self) -> bool { !self.inner_info.is_all_prepared() } 

    fn port_name_handle(&mut self, port_names: &std::collections::HashMap<String, usize>) -> Option<CompilerErrorType> {
        for (name, port_num) in port_names {
            if let Some(x) = PrPrepared::from_name(name) {
                if *port_num >= self.max_port_amount { 
                    return Some( 
                        CompilerErrorType::Other(
                            format!("too big port num({}), max is {}", port_num, self.max_port_amount)
                        ) 
                    ) 
                }
                self.cgen_set_port(&x, *port_num);
                self.inner_info.set_prepared(x);
            }
        }
        
        None
    }
}