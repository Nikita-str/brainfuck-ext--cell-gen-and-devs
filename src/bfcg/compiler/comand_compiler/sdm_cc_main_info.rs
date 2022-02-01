use crate::bfcg::compiler::compiler_pos::CompilerPos;

use super::std_dir_mem_cc::MAX_PR;


#[derive(Clone)]
pub(in crate) struct SdmCcOpenWhile{
    pub compiler_pos: CompilerPos,
    pub cmd_pos: usize,
}

impl SdmCcOpenWhile{
    pub fn new(compiler_pos: CompilerPos, cmd_pos: usize) -> Self { Self{ compiler_pos, cmd_pos } }
}

#[derive(Clone)]
pub(in crate) struct SdmCcMainInfo{
    pub open_while: Vec<SdmCcOpenWhile>,
    /// changed after CUR:SE on SE, this need for
    ///   
    /// ```.,..@,.rrw```
    /// 
    /// compiled into
    /// 
    /// ``` CUR[CONSOLE_PR] SET[console_port] WR READ WR WR CUR[WIN_PR] SET[win_port] CUR[CONSOLE_PR] READ WR CUR[USER_PR] READ READ WR```
    pub cur_port_reg: usize,
    pub cem_cur_cell_in_reg: bool,
    pub max_port_amount: usize,
    pub max_jump_size: usize,
}

impl SdmCcMainInfo {
    pub fn new(max_port_amount: usize, max_jump_size: usize) -> Self{
        Self{
            open_while: vec![],
            cur_port_reg: MAX_PR, // invalid value
            cem_cur_cell_in_reg: false,
            max_port_amount,
            max_jump_size,
        }
    }

    pub fn open_while_amount(&self) -> usize { self.open_while.len() }
    pub fn set_cur_pr_invalid(&mut self) { self.cur_port_reg = MAX_PR; }
    pub fn set_cur_pr(&mut self, cur_port_reg: usize) { self.cur_port_reg = cur_port_reg; }

    pub fn get_cur_pr(&self) -> usize { self.cur_port_reg }
    pub fn get_max_jump_size(&self) -> usize { self.max_jump_size }
    pub fn get_max_port_amount(&self) -> usize { self.max_port_amount }

    pub fn can_be_the_same_compilation(a: &Self, b: &Self) -> bool {
        a.max_jump_size == b.max_jump_size && a.max_port_amount == b.max_port_amount
    }
}

impl Default for SdmCcMainInfo {
    fn default() -> Self {
        Self { 
            open_while: vec![], 
            cur_port_reg: MAX_PR, 
            cem_cur_cell_in_reg: false, 
            max_port_amount: 0, 
            max_jump_size: 0 
        }
    }
}