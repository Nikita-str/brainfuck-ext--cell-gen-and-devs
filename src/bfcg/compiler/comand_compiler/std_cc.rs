use std::collections::LinkedList;
use crate::bfcg::{
    general::{se_fn::std_se_encoding, self}, 
    compiler::{compiler_pos::CompilerPos, compiler_error::CompilerErrorType, code_gen::{self, cgen_set_cell_value}, valid_cmd::ValidCMD}, 
    dev_emulators::dev_utilities::{mem_dev::{CellMemDevStartAction, CmdMemDevStartAction}, win_dev::WinDevStartAction}, 
    vm::hardware_info::HardwareInfo
};
use super::{
    CmdCompiler, PortNameHandler, 
    std_cc_additional_info::{StdCcAditionalInfo, PrPrepared}, 
    to_u8_seq::ToU8Seq, 
    std_cc_main_info::{StdCcMainInfo, StdCcOpenWhile}
};

fn one_ll(x: u8) -> LinkedList<u8> { 
    let mut ret = LinkedList::new();
    ret.push_back(x);
    ret
 }

 fn add_front_ll(mut ll: LinkedList<u8>, front_value: u8) -> LinkedList<u8>{
    ll.push_front(front_value);
    ll
 }

pub enum StdCmdNames{
    Pass, // 0x00
    Test, // 0x01
    Cur(usize), // 0x02:SE
    Set, // 0x03 + CEM: SE
    Read, // 0x04
    Write, // 0x05
    //SetRegConst(u8), // 0x06
}

impl StdCmdNames {
    /// use carefully! valid only name of instruction, not inner value
    pub fn is_start_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::Pass), 
            0x01 => Some(Self::Test), 
            0x02 => Some(Self::Cur(0xBAD_C0DE)), 
            0x03 => Some(Self::Set), 
            0x04 => Some(Self::Read), 
            0x05 => Some(Self::Write), 
            //0x06 => Some(Self::SetRegConst(0xBA)),
            _ => None 
        }
    }
}

impl ToU8Seq<<LinkedList<u8> as IntoIterator>::IntoIter> for StdCmdNames {
    fn to_u8_seq(&self) -> <LinkedList<u8> as IntoIterator>::IntoIter {
        match self {
            Self::Pass => one_ll(0x00).into_iter(),
            Self::Test => one_ll(0x01).into_iter(),
            Self::Cur(se) => add_front_ll(std_se_encoding(*se), 0x02).into_iter(),
            Self::Set => one_ll(0x03).into_iter(),
            Self::Read => one_ll(0x04).into_iter(),
            Self::Write => one_ll(0x05).into_iter(),
            //Self::SetRegConst(byte) => add_front_ll(one_ll(*byte), 0x06).into_iter(),
        }
    }
}


#[derive(Clone, Copy)]
#[repr(u8)]
pub enum RegCmdNames{
    Zero = 0x06,
    TestZero = 0x07,
    Inc = 0x08,
    Dec = 0x09,
    LeftShift = 0x0A,
    RightShift = 0x0B,
    And = 0x0C,
    Bnd = 0x0D,
}

impl RegCmdNames {
    pub fn try_from_byte(byte: u8) -> Option<Self> { // need macros, but .. not now
        match byte {
            x if x == Self::Zero as u8 => { Some(Self::Zero) }
            x if x == Self::TestZero as u8 => { Some(Self::TestZero) }
            x if x == Self::Inc as u8 => { Some(Self::Inc) }
            x if x == Self::Dec as u8 => { Some(Self::Dec) }
            x if x == Self::LeftShift as u8 => { Some(Self::LeftShift) }
            x if x == Self::RightShift as u8 => { Some(Self::RightShift) }
            x if x == Self::And as u8 => { Some(Self::And) }
            x if x == Self::Bnd as u8 => { Some(Self::Bnd) }
            _ => None
        }
    }
}

impl ToU8Seq<<LinkedList<u8> as IntoIterator>::IntoIter> for RegCmdNames {
    fn to_u8_seq(&self) -> <LinkedList<u8> as IntoIterator>::IntoIter { one_ll(*self as u8).into_iter() }
}

impl RegCmdNames{
    pub fn try_from_valid_cmd(valid_cmd: ValidCMD) -> Option<Self> {
        match valid_cmd {
            ValidCMD::ZeroedCell => Some(Self::Zero),
            ValidCMD::IncValue => Some(Self::Inc),
            ValidCMD::DecValue => Some(Self::Dec),
            ValidCMD::LeftShift => Some(Self::LeftShift),
            ValidCMD::RightShift => Some(Self::RightShift),
            ValidCMD::And => Some(Self::And),
            ValidCMD::Bnd => Some(Self::Bnd),
            ValidCMD::TestZeroCell => Some(Self::TestZero),
            _ => None,
        }
    }
}

/// PR is PORT REG
const USER_PR: usize = 0;
pub const MEM_CELL_PR: usize = USER_PR + 1;
pub const MEM_CMD_PR: usize = MEM_CELL_PR + 1;
const CONSOLE_PR: usize = MEM_CMD_PR + 1;
const WIN_PR: usize = CONSOLE_PR + 1;
pub(in super) const MAX_PR: usize = WIN_PR + 1;

pub const MIN_PORT_AMOUNT: usize = MAX_PR - 1;

/// standart cc with cpu direct memory access (Inc, Dec, NextCell, JumpRight, ...)
pub struct StdCmdCompiler{
    program: Vec<u8>,
    main_info: StdCcMainInfo,
    inner_info: StdCcAditionalInfo,
}

impl StdCmdCompiler{

    /// set main info in most unknown state & return real cur state
    fn clear_main_info(&mut self) -> StdCcMainInfo { 
        let max_port_amount = self.main_info.max_port_amount;
        let max_jump_size = self.main_info.max_jump_size;

        let save_main_info = std::mem::take(&mut self.main_info);
        
        self.main_info.max_jump_size = max_jump_size;
        self.main_info.max_port_amount = max_port_amount;
        
        save_main_info
    }

    /// load save for main info
    /// ## painc
    /// if `save_main_info` was getted not by `self.clear_main_info()`
    fn load_main_info(&mut self, mut save_main_info: StdCcMainInfo) { 
        if !StdCcMainInfo::can_be_the_same_compilation(&self.main_info, &save_main_info) { panic!("bad save_main_info") }
        std::mem::swap(&mut self.main_info, &mut save_main_info);
    }


    /// amount of byte that max need for compile cmd_seq
    /// ## panic
    /// if it cant be compiled
    fn reserve_cmd_seq(&mut self, cmd_seq: &str) -> usize{
        self.clear_main_info(); // ignore save
        let mut save_program = std::mem::take(&mut self.program);
        let mut len = 0; // amount of byte that max need for setting cell value 
        for cmd in cmd_seq.chars() {
            if let Some(_) = self.cmd_compile(cmd, CompilerPos::new()) {
                panic!("cant compile cmd_seq (bad char: {}) :/", cmd)
            }
        }
        len += self.program.len();
        if self.main_info.open_while.len() > 0 { panic!("potential [ALGO ERROR] or cmd_seq is not full") }
        self.program.clear();
        if save_program.len() > 0 { self.program = std::mem::take(&mut save_program); }
        len
    }

    /// amount of byte that max need for compile one_cmd
    /// ## panic
    /// if it cant be compiled
    fn reserve_cmd(&mut self, one_cmd: ValidCMD) -> usize{
        self.clear_main_info(); // ignore save
        let mut save_program = std::mem::take(&mut self.program);
        let cmd = one_cmd.std_to_char();
        if let None = self.cmd_compile(cmd, CompilerPos::new()) {
            let ret_len = self.program.len();
            if save_program.len() > 0 { self.program = std::mem::take(&mut save_program); }
            return ret_len
        } else {
            panic!("cant compile cmd_seq (bad char: {}) :/", cmd)
        }
    }

    /// ### Ru comment version:
    /// вычисляем размер (в байтах) который нужно оставить 
    /// для генерации в CEM SE последовательности для 
    /// числа не больше max_x 
    fn reserve_prog_space_cem_se_gen(&mut self, max_x: usize) -> usize{
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
    fn reserve_one_pr_init(&mut self) -> usize {
        let first_sh = self.reserve_cmd(ValidCMD::NextCell);
        let se_gen = self.reserve_prog_space_cem_se_gen(self.main_info.get_max_port_amount() - 1);
        let to_start = self.reserve_cmd_seq(&code_gen::cgen_move_to_next_after_left_zero());
        let cur_and_set = StdCmdNames::Cur(MAX_PR).to_u8_seq().count() + StdCmdNames::Set.to_u8_seq().count();
        let to_end = self.reserve_cmd_seq(&code_gen::cgen_move_to_prev_before_right_zero());
        let clear = self.reserve_cmd_seq(&code_gen::cgen_zero_while_not_zero());

        let byte_len = first_sh + se_gen + to_start + cur_and_set + to_end + clear;
        byte_len
    }

    fn reserve_initial_program_space(&mut self) -> Vec<u8>{
        // CUR[X_PR] SET[Z] where Z < max_port_amount
        let one_pr_sz = 
            if self.inner_info.get_pr_reserve_sz() == 0 { self.reserve_one_pr_init() } 
            else { self.inner_info.get_pr_reserve_sz() };
            
        let mut initial_pass = vec![];
        for _ in 0..(one_pr_sz * (MAX_PR - 1 - 2)) { // - 1 cause USER_PR not need to set; - 2 cause COM & CEM default-setted by hardware
            for x in StdCmdNames::Pass.to_u8_seq() { initial_pass.push(x) }
        }
        initial_pass
    }

    fn program_init(&mut self) -> Vec<u8> {
        self.program.clear();
        self.main_info.open_while.clear();

        let mut ret = self.reserve_initial_program_space();

        let cur_port_reg = 0;
        for x in StdCmdNames::Cur(cur_port_reg).to_u8_seq() { ret.push(x); }
        for x in StdCmdNames::Set.to_u8_seq() { ret.push(x); }

        self.cancel_pseudo(Some(cur_port_reg));        
        ret
    }

    fn cgen_set_port(&mut self, pr: &PrPrepared, port_num: usize) {
        if pr == &PrPrepared::MemCell || pr == &PrPrepared::MemCmd { return }

        let save_main_info = self.clear_main_info();

        let pr_index = pr.to_index();
        let set_byte = move |x: &mut Self, cgen_compiled_sz: &mut usize, byte|{
            x.program[pr_index * x.inner_info.get_pr_reserve_sz() + *cgen_compiled_sz] = byte;
            *cgen_compiled_sz += 1;
        };
        let cgen_compile = |cgen: &str, mut x: Self, cgen_compiled_sz: &mut usize|{
            let mut save_program = std::mem::take(&mut x.program);
            for cmd in cgen.chars() {
                if let Some(_) = x.cmd_compile(cmd, CompilerPos::new()) {
                    panic!("cant compile auto gen code")
                }
            }
            std::mem::swap(&mut x.program, &mut save_program);
            let bytes = save_program;
            for byte in bytes { set_byte(&mut x, cgen_compiled_sz, byte); } 
            return x;
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

        let self_take = std::mem::take(self);
        let mut new_self = cgen_compile(&cgen, self_take, &mut cgen_compiled_sz);
        std::mem::swap(self, &mut new_self);

        // #############################################
        // CGEN[CENTER]    (CUR + SET)
        let cur = 
            match pr {
                PrPrepared::Console => CONSOLE_PR,
                PrPrepared::Win => WIN_PR,
                PrPrepared::MemCell => MEM_CELL_PR,
                PrPrepared::MemCmd => MEM_CMD_PR,
            };
        for byte in StdCmdNames::Cur(cur).to_u8_seq() { set_byte(self, &mut cgen_compiled_sz, byte); }
        for byte in StdCmdNames::Set.to_u8_seq() { set_byte(self, &mut cgen_compiled_sz, byte); }

        // #############################################
        // CGEN[END]   
        cgen.clear();
        code_gen::add_cgen_move_to_prev_before_right_zero(&mut cgen);
        code_gen::add_cgen_zero_while_not_zero(&mut cgen);
        let self_take = std::mem::take(self);
        let mut new_self = cgen_compile(&cgen, self_take, &mut cgen_compiled_sz);
        std::mem::swap(self, &mut new_self);

        if cgen_compiled_sz > self.inner_info.get_pr_reserve_sz() { panic!("[ALGO ERROR]: wrong reserved size counted") }

        // nullify if already use this cell:
        for _ in 0..(self.inner_info.get_pr_reserve_sz() - cgen_compiled_sz) {
            set_byte(self, &mut cgen_compiled_sz, 0x00);
        }

        if cgen_compiled_sz != self.inner_info.get_pr_reserve_sz() { panic!("[ALGO ERROR] :/") }

        self.load_main_info(save_main_info);
    }

    fn cancel_pseudo(&mut self, cur_port_reg: Option<usize>){
        if !self.program.is_empty() { panic!("potential ALGO ERROR") }
        if self.main_info.open_while_amount() != 0 { panic!("open while amount must be zero when pseudo") }
        if let Some(cur_port_reg) = cur_port_reg { 
            self.main_info.set_cur_pr(cur_port_reg);
        } else {
            self.main_info.set_cur_pr_invalid();
        }
        self.main_info.cem_cur_cell_in_reg = false;
    }

    fn get_cur_pr(&self) -> usize { self.main_info.get_cur_pr() }
    fn set_cur_pr(&mut self, cur_pr: usize) { self.main_info.set_cur_pr(cur_pr) }

    fn get_cem_cur_cell_in_reg(&self) -> bool { self.main_info.cem_cur_cell_in_reg }
    fn set_cem_cur_cell_in_reg(&mut self, new_value: bool) { self.main_info.cem_cur_cell_in_reg = new_value; }

    fn get_jump_pass_amount(&self) -> usize {
        let pass_amount = self.inner_info.get_jump_pass_amount();
        if pass_amount != 0 { pass_amount }
        else { std_se_encoding(self.main_info.get_max_jump_size()).len() }
    }

    /// ## params
    /// + max_jump_size: if you dont know => use memory size
    pub fn new(hardware_info: &HardwareInfo) -> Self{
        if hardware_info.max_port_amount < MIN_PORT_AMOUNT { panic!("no enough port for all std devs (need minimum ports for CEM, COM, console & win)") }
        let mut ret = Self{ 
            program: vec![],//Self::program_init(max_port_amount),
            main_info: StdCcMainInfo::new(hardware_info.max_port_amount, hardware_info.max_jump_size),
            inner_info: StdCcAditionalInfo::new(),
        };

        let pr_res_sz = ret.reserve_one_pr_init();
        ret.inner_info.set_pr_reserve_sz(pr_res_sz);
        ret.inner_info.set_jump_pass_amount(ret.get_jump_pass_amount());
        
        ret.program = ret.program_init();
        ret
    }
}

// ------------------------------------------
// + [COMPILE TO BYTE] 
impl StdCmdCompiler{

    #[inline]
    fn ctb_const_write(&mut self, x: u8) {
        self.ctb_to(RegCmdNames::Zero);
        for c in cgen_set_cell_value(x, false).chars() {
            match c {
                x if ValidCMD::std_cmd_to_char(ValidCMD::IncValue) == x => {
                    self.ctb_to(RegCmdNames::try_from_valid_cmd(ValidCMD::IncValue).unwrap())
                }
                x if ValidCMD::std_cmd_to_char(ValidCMD::LeftShift) == x => {
                    self.ctb_to(RegCmdNames::try_from_valid_cmd(ValidCMD::LeftShift).unwrap())
                }
                _ => { panic!() }
            }
        } 
        self.ctb_to(StdCmdNames::Write);
    }

    #[inline]
    fn ctb_to<Iter>(&mut self, to_u8_seq: impl ToU8Seq<Iter>) 
    where Iter: Iterator<Item = u8>
    {
        for x in to_u8_seq.to_u8_seq() { self.program.push(x); }
    }

    #[inline]
    fn ctb_set_cur_pr(&mut self, new_cur_pr: usize) {
        if new_cur_pr == self.get_cur_pr() { return }
        if new_cur_pr >= MAX_PR { panic!("ALGO ERROR: invalid port reg") }
        self.ctb_to(StdCmdNames::Cur(new_cur_pr));
        self.set_cur_pr(new_cur_pr);
    }

    #[inline]
    fn ctb_load_cur_cem(&mut self) {
        if self.get_cem_cur_cell_in_reg() { return }
        self.ctb_set_cur_pr(MEM_CELL_PR);
        self.ctb_const_write(CellMemDevStartAction::GetCellValue as u8);
        self.ctb_to(StdCmdNames::Read);
        self.set_cem_cur_cell_in_reg(true);
    }

    #[inline]
    fn ctb_unload_cur_cem(&mut self) {
        if !self.get_cem_cur_cell_in_reg() { return }
        self.ctb_set_cur_pr(MEM_CELL_PR);
        self.ctb_const_write(CellMemDevStartAction::SetCellValue as u8); 
        self.ctb_to(StdCmdNames::Write);
    }

}
// - [COMPILE TO BYTE] 
// ------------------------------------------

impl StdCmdCompiler {
    fn cmd_compile_inner(&mut self, valid_cmd: ValidCMD, pos: CompilerPos) -> Option<CompilerErrorType> {

        if let Some(reg_cmd) = RegCmdNames::try_from_valid_cmd(valid_cmd.clone()) {
            self.ctb_load_cur_cem();
            self.ctb_to(reg_cmd);
            return None
        }

        // {*1}: we can not unload value cause it will rewrite by this op
        // {*2}: cur value must be in reg
        
        match valid_cmd {
            ValidCMD::NextCell | ValidCMD::PrevCell | ValidCMD::CreateCell | ValidCMD::DeleteCell => {
                self.ctb_unload_cur_cem();
                
                self.ctb_set_cur_pr(MEM_CELL_PR);
                self.ctb_const_write(CellMemDevStartAction::from_valid_cmd(&valid_cmd) as u8);

                if let ValidCMD::CreateCell = valid_cmd {
                    self.ctb_to(RegCmdNames::Zero);
                    self.set_cem_cur_cell_in_reg(true);
                } else {
                    self.set_cem_cur_cell_in_reg(false);
                }
            }
            ValidCMD::DecCoordX | ValidCMD::DecCoordY | ValidCMD::IncCoordX | ValidCMD::IncCoordY | ValidCMD::RedrawWin => {
                self.ctb_set_cur_pr(WIN_PR);
                self.ctb_const_write(WinDevStartAction::from_valid_cmd(&valid_cmd) as u8);
            }
            ValidCMD::SetWinValue => {
                self.ctb_set_cur_pr(WIN_PR);
                self.ctb_const_write(WinDevStartAction::from_valid_cmd(&valid_cmd) as u8);
                
                self.ctb_load_cur_cem(); // {*2}
                self.ctb_to(StdCmdNames::Write);
            }
            // #############################################################
            ValidCMD::PrintValue => {
                self.ctb_load_cur_cem(); // {*2}
                self.ctb_set_cur_pr(CONSOLE_PR);
                self.ctb_to(StdCmdNames::Write);
            }
            ValidCMD::ReadValue => {
                // check {*1}
                self.ctb_set_cur_pr(CONSOLE_PR);
                self.ctb_to(StdCmdNames::Read);
                self.set_cem_cur_cell_in_reg(true);
            }
            // #############################################################
            ValidCMD::TestPort => {
                // check {*1}
                self.ctb_set_cur_pr(USER_PR);
                self.ctb_to(StdCmdNames::Test);
                self.set_cem_cur_cell_in_reg(true);
            }
            ValidCMD::ReadFromPort => {
                // check {*1}
                self.ctb_set_cur_pr(USER_PR);
                self.ctb_to(StdCmdNames::Read);
                self.set_cem_cur_cell_in_reg(true);
            }
            ValidCMD::WriteIntoPort => {
                self.ctb_load_cur_cem(); // {*2}
                self.ctb_set_cur_pr(USER_PR);
                self.ctb_to(StdCmdNames::Write);
            }
            ValidCMD::SetCurPort => { // TODO:??? all ok ???
                self.ctb_unload_cur_cem();
                self.ctb_set_cur_pr(USER_PR);
                self.ctb_to(StdCmdNames::Set);
            }
            // #############################################################
            ValidCMD::StartWhileNZ => {
                self.ctb_unload_cur_cem(); // cause it's value can be changed
                
                self.ctb_set_cur_pr(MEM_CMD_PR);
                self.ctb_const_write(CmdMemDevStartAction::JumpForward as u8);
                
                self.ctb_load_cur_cem(); // cause we need it's value in reg
                self.ctb_set_cur_pr(MEM_CMD_PR);
                self.ctb_to(StdCmdNames::Write); // value for jump test (if 0 => jump)
                
                let cmd_pos = self.program.len();
                self.main_info.open_while.push(StdCcOpenWhile::new(pos, cmd_pos));

                // reserve space in COM for jump:
                for _ in 0..self.get_jump_pass_amount() { self.ctb_to(StdCmdNames::Pass); }
            }
            ValidCMD::EndWhileNZ => {
                if self.main_info.open_while.is_empty() { return Some(CompilerErrorType::ClosedWhileWithoutOpen) }

                self.ctb_unload_cur_cem(); // cause it's value can be changed
                
                self.ctb_set_cur_pr(MEM_CMD_PR);
                self.ctb_const_write(CmdMemDevStartAction::JumpBackward as u8);

                self.ctb_load_cur_cem(); // cause we need it's value in reg
                self.ctb_set_cur_pr(MEM_CMD_PR);
                self.ctb_to(StdCmdNames::Write); // value for jump test (if !0 => jump)

                let cmd_pos = self.program.len();
                let prev_open = self.main_info.open_while.pop().unwrap();

                let mut backward_shift_cmd_len = 0;
                let backward_jump = cmd_pos - prev_open.cmd_pos + self.get_jump_pass_amount();
                for x in std_se_encoding(backward_jump) { 
                    self.program.push(x); 
                    backward_shift_cmd_len += 1;
                }

                let forward_jump = backward_jump + backward_shift_cmd_len;
                if forward_jump > self.main_info.get_max_jump_size() { return Some(CompilerErrorType::Other("too big jump".to_owned())) }
                for (ind, x) in std_se_encoding(forward_jump).into_iter().enumerate() { 
                    let value_must_0 = std::mem::replace(&mut self.program[prev_open.cmd_pos + ind], x);
                    if  value_must_0 != 0x00 { panic!("[ALGO ERROR]: must be 0; index = {}; value = {}", ind, value_must_0) }
                }
            }
            // #############################################################
            ValidCMD::Clone => {
                self.ctb_unload_cur_cem();
                self.ctb_load_cur_cem(); // {*2}
                let res = self.cmd_compile_inner(ValidCMD::NextCell, pos);
                if res.is_some() { return res }
            }
            ValidCMD::ZeroedCell => {
                self.ctb_load_cur_cem();
                self.ctb_to(RegCmdNames::Zero);
            }
            // #############################################################
            _ => panic!("unaccounted cmd {}", valid_cmd.std_to_char()),
        }   
             
        // SOLVED : in get_program() ... self.ctb_unload_cur_cem();
        // 0x00:
        //       in end of program we can have { cem_cur_cell_in_reg == true }
        //       so we need to self.ctb_unload_cur_cem() in end of program
        //       ---
        //       we can do this by adding '> <' in the end of code

        None
    }
}
    
impl CmdCompiler<u8> for StdCmdCompiler{
    fn cmd_compile(&mut self, cmd: char, pos: CompilerPos) -> Option<CompilerErrorType> {
        let valid_cmd = ValidCMD::std_parse_char(cmd);
        if valid_cmd.is_none() { return Some(CompilerErrorType::UnknownCmd(cmd)) }
        let valid_cmd = valid_cmd.unwrap();
        
        self.cmd_compile_inner(valid_cmd, pos)
    }

    fn get_program(mut self) -> Result<Vec<u8>, CompilerErrorType> {
        self.ctb_unload_cur_cem(); // cause we can have value in reg, if we want to have the same value in CEM as if it was just bf 

        if !self.main_info.open_while.is_empty() {
            Err(CompilerErrorType::NotClosedWhile(self.main_info.open_while.into_iter().map(|x|x.compiler_pos).collect()))
        } else {
            Ok(self.program)
        }
    }
}

impl PortNameHandler for StdCmdCompiler{
    fn need_port_name_handle(&self) -> bool { !self.inner_info.is_all_prepared() } 

    fn port_name_handle(&mut self, port_names: &std::collections::HashMap<String, usize>) -> Option<CompilerErrorType> {
        for (name, port_num) in port_names {
            if let Some(x) = PrPrepared::from_name(name) {
                if *port_num >= self.main_info.get_max_port_amount() { 
                    return Some( 
                        CompilerErrorType::Other(
                            format!("too big port num({}), max is {}", port_num, self.main_info.get_max_port_amount())
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

impl Default for StdCmdCompiler{
    fn default() -> Self {
        Self { 
            program: Default::default(), 
            main_info: Default::default(),
            inner_info: Default::default() 
        }
    }
}