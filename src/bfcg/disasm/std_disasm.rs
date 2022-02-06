use std::collections::HashMap;
use crate::bfcg::{dev_emulators::{std_dev::StdDevName, dev_utilities::{win_dev::WinDevStartAction, mem_dev::{CmdMemDevStartAction, CellMemDevStartAction}}}, general::se_fn::{MIN_BIG_BYTE, std_se_decoding}};

pub struct StdDisasmInfo{
    dev_to_port_reg: HashMap<StdDevName, usize>,
    port_reg_to_dev: HashMap<usize, StdDevName>,
}

impl StdDisasmInfo {
    pub fn new() -> Self{ 
        Self{ 
            dev_to_port_reg: HashMap::new(),
            port_reg_to_dev: HashMap::new(),
        } 
    }

    pub fn std_init(&mut self) {
        self.add_dev_to_pr(StdDevName::CellMem, 0x01);
        self.add_dev_to_pr(StdDevName::CmdMem, 0x02);
        self.add_dev_to_pr(StdDevName::Console, 0x03);
        self.add_dev_to_pr(StdDevName::Win, 0x04);
    }

    pub fn get_dev_to_pr(&self) -> &HashMap<StdDevName, usize> { &self.dev_to_port_reg }
    pub fn get_pr_to_dev(&self) -> &HashMap<usize, StdDevName> { &self.port_reg_to_dev }

    pub fn add_dev_to_pr(&mut self, dev: StdDevName, port_reg: usize) {
        if let Some(_) = self.dev_to_port_reg.insert(dev, port_reg) { panic!("device already used") }
        if let Some(_) = self.port_reg_to_dev.insert(port_reg, dev) { panic!("port reg already used") }
    }
}

/// !!! HERE BAD CODE !!!
///  
/// done in a harry, here exist unnamed const, copypast & so on ...
pub fn std_disasm<'a, Iter: Iterator<Item = &'a u8>>(program: Iter, disasm_info: &StdDisasmInfo) -> Result<String, String> {
    let mut ret = String::new();

    let mut pr_index_to_name = HashMap::new();
    let mut dev_action = HashMap::new();
    for (dev, pr) in disasm_info.get_dev_to_pr() {
        match dev {
            StdDevName::Console => {
                ret.push_str(&format!("CONST CONSOLE_PR = {}\n", pr));
                pr_index_to_name.insert(*pr, "CONSOLE_PR");
            }
            StdDevName::Win => {
                ret.push_str(&format!("CONST WIN_PR = {}\n", pr));
                pr_index_to_name.insert(*pr, "WIN_PR");

                ret.push_str(&format!("CONST WIN_DEC_X = {}\n", WinDevStartAction::DecCoordX as usize));
                ret.push_str(&format!("CONST WIN_DEC_Y = {}\n", WinDevStartAction::DecCoordY as usize));
                ret.push_str(&format!("CONST WIN_INC_X = {}\n", WinDevStartAction::IncCoordX as usize));
                ret.push_str(&format!("CONST WIN_INC_Y = {}\n", WinDevStartAction::IncCoordY as usize));
                ret.push_str(&format!("CONST WIN_REDRAW = {}\n", WinDevStartAction::RedrawWin as usize));
                ret.push_str(&format!("CONST WIN_SET_VAL = {}\n", WinDevStartAction::SetWinValue as usize));

                dev_action.insert((StdDevName::Win, WinDevStartAction::DecCoordX as usize), "WIN_DEC_X");
                dev_action.insert((StdDevName::Win, WinDevStartAction::DecCoordY as usize), "WIN_DEC_Y");
                dev_action.insert((StdDevName::Win, WinDevStartAction::IncCoordX as usize), "WIN_INC_X");
                dev_action.insert((StdDevName::Win, WinDevStartAction::IncCoordY as usize), "WIN_INC_Y");
                dev_action.insert((StdDevName::Win, WinDevStartAction::RedrawWin as usize), "WIN_REDRAW");
                dev_action.insert((StdDevName::Win, WinDevStartAction::SetWinValue as usize), "WIN_SET_VAL");
            }
            StdDevName::CmdMem => {
                ret.push_str(&format!("CONST COM_PR = {}\n", pr));
                pr_index_to_name.insert(*pr, "COM_PR");

                ret.push_str(&format!("CONST COM_JMP_F = {}\n", CmdMemDevStartAction::JumpForward as usize));
                ret.push_str(&format!("CONST COM_JMP_B = {}\n", CmdMemDevStartAction::JumpBackward as usize));

                dev_action.insert((StdDevName::CmdMem, CmdMemDevStartAction::JumpForward as usize), "COM_JMP_F");
                dev_action.insert((StdDevName::CmdMem, CmdMemDevStartAction::JumpBackward as usize), "COM_JMP_B");
            }
            StdDevName::CellMem => {
                ret.push_str(&format!("CONST CEM_PR = {}\n", pr));
                pr_index_to_name.insert(*pr, "CEM_PR");
                
                ret.push_str(&format!("CONST CEM_GET_VAL = {}\n", CellMemDevStartAction::GetCellValue as usize));
                ret.push_str(&format!("CONST CEM_SET_VAL = {}\n", CellMemDevStartAction::SetCellValue as usize));
                ret.push_str(&format!("CONST CEM_PREV = {}\n", CellMemDevStartAction::PrevCell as usize));
                ret.push_str(&format!("CONST CEM_NEXT = {}\n", CellMemDevStartAction::NextCell as usize));
                ret.push_str(&format!("CONST CEM_CR = {}\n", CellMemDevStartAction::CreateCell as usize));
                ret.push_str(&format!("CONST CEM_DEL = {}\n", CellMemDevStartAction::DeleteCell as usize));

                dev_action.insert((StdDevName::CellMem, CellMemDevStartAction::GetCellValue as usize), "CEM_GET_VAL");
                dev_action.insert((StdDevName::CellMem, CellMemDevStartAction::SetCellValue as usize), "CEM_SET_VAL");
                dev_action.insert((StdDevName::CellMem, CellMemDevStartAction::PrevCell as usize), "CEM_PREV");
                dev_action.insert((StdDevName::CellMem, CellMemDevStartAction::NextCell as usize), "CEM_NEXT");
                dev_action.insert((StdDevName::CellMem, CellMemDevStartAction::CreateCell as usize), "CEM_CR");
                dev_action.insert((StdDevName::CellMem, CellMemDevStartAction::DeleteCell as usize), "CEM_DEL");
            }
        }
    }

    let mut iter = program;
    let mut cur_dev = None;

    let mut swap = false;
    let mut swap_value = 0u8;
    let mut it_is_swap_act = false;
    let mut first_swap = true;
    
    loop {
        let cmd = iter.next();
        let cmd = if let Some(cmd) = cmd { *cmd } else { break; };

        match cmd {
            0x00 => ret.push_str("PASS"),
            0x01 => ret.push_str("TEST"),

            0x02 => {
                ret.push_str("CUR[");
                let mut se = vec![];
                loop {
                    let cmd = iter.next();
                    let cmd = if let Some(cmd) = cmd { *cmd } else { return Err(format!("program ended on CUR")) };
                    se.push(cmd);
                    if cmd < MIN_BIG_BYTE { break; }
                }
                let port_reg = std_se_decoding(se.iter());  
                let port_reg = if let Some(port_reg) = port_reg { port_reg } else { return Err(format!("bad se seq in CUR")) };
                if let Some(dev_const) = pr_index_to_name.get(&port_reg) {  
                    ret.push_str(dev_const);
                    cur_dev = Some(disasm_info.get_pr_to_dev().get(&port_reg).unwrap().clone());
                } else { 
                    ret.push_str(&port_reg.to_string());
                    cur_dev = None;
                }
                ret.push_str("]\n");
            }

            0x03 => ret.push_str("SET"),
            0x04 => ret.push_str("READ"),
            0x05 => ret.push_str("WR"),
            
            0x06 => {
                ret.push_str("SWAP ");
                swap = !swap;
                if swap { it_is_swap_act = true; }
                if !swap {
                    if let Some(dev) = cur_dev{
                        if let Some(act) = dev_action.get(&(dev, swap_value as usize)) {
                            ret.push_str(&format!(" ;; CWR[{}]", act));
                        } else {
                            ret.push_str(&format!(" ;; CWR[{}]", swap_value));
                        }
                    } else {
                        ret.push_str(&format!(" ;; CWR[{}]", swap_value));
                    } 

                    if first_swap {
                        ret.push_str("     {*1}: if compiled by std compiler => this is CWR cmd ([C]onst [WR]ite)");
                    } else {
                        ret.push_str("      (check {*1})");
                    }
                    first_swap = false;
                }
            }
            
            0x07 => ret.push_str("TZ"),
            0x08 => {
                ret.push_str("INC");
                if swap { 
                    swap_value = u8::overflowing_add(swap_value, 1).0; 
                    it_is_swap_act = true;
                }
            }
            0x09 => ret.push_str("DEC"),
            0x0A => {
                ret.push_str("LSH");
                if swap { 
                    swap_value <<= 1; 
                    it_is_swap_act = true;
                }
            }
            0x0B => ret.push_str("RSH"),
            0x0C => ret.push_str("AND"),
            0x0D => ret.push_str("BND"),
            0x0E => {
                ret.push_str("ZER");
                if swap { 
                    swap_value = 0; 
                    it_is_swap_act = true;
                }
            }
            
            _ => ret.push_str(&format!("![{}]", cmd)),
            // _ => return Err(format!("unknown cmd byte {}", cmd)),
        }

        if swap && !it_is_swap_act { swap = false; ret.push_str(" ;; ! sorry, maybe they shouldn't be on the same line "); }

        ret.push(if swap { ' ' } else { '\n' });
    }

    Ok(ret)
}
