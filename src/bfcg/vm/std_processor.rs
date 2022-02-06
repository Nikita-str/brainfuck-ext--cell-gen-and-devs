use std::collections::HashMap;
use crate::bfcg::dev_emulators::dev::Dev;
use crate::bfcg::compiler::comand_compiler::{MIN_PORT_AMOUNT, MEM_CMD_PR, MEM_CELL_PR, RegCmdNames, StdCmdNames};
use crate::bfcg::dev_emulators::dev_utilities::mem_dev::CellMemDevStartAction;
use crate::bfcg::general::se_fn::{MIN_BIG_BYTE, std_se_decoding};

pub struct StdProcessor<'a> {
    // if need impl parallel work with the same dev by dif processor 
    // then need use Rc<RefCell<dyn Dev>>
    devs: HashMap<usize, Box<dyn Dev + 'a>>,
    port_amount: usize,

    byte_await: Option<Vec<u8>>,

    main_regs: [u8; MR_AMOUNT],
    reg_cur_mr: usize,

    port_regs: [usize; PR_AMOUNT],
    reg_cur_pr: usize,
}

const MR_AMOUNT:usize = 2;

const PR_AMOUNT:usize = MIN_PORT_AMOUNT + 1;
const PR_COM:usize = MEM_CMD_PR;
const PR_CEM:usize = MEM_CELL_PR;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] INIT 
impl<'a> StdProcessor<'a> {
    pub fn new(port_amount: usize, pr_com: usize, pr_cem: usize) -> Self {
        let mut port_regs = [0; PR_AMOUNT];
        port_regs[PR_COM] = pr_com;
        port_regs[PR_CEM] = pr_cem;

        Self { 
            devs: HashMap::new(),
            port_amount,

            byte_await: None,

            main_regs: [0; 2],
            reg_cur_mr: 0,

            port_regs,
            reg_cur_pr: 0,
        }
    }

    pub fn add_boxed_device(&mut self, dev: Box<dyn Dev + 'a>, port: usize) -> Result<AddDeviceOk, ()> {
        if port >= self.port_amount { return Err(()) }

        if let Some(_) = self.devs.insert(port, dev) { 
            Ok(AddDeviceOk::OldDevDisconected)
        } else {
            Ok(AddDeviceOk::Ok)
        }        
    }

    pub fn add_device<D: 'a + Dev>(&mut self, dev: D, port: usize) -> Result<AddDeviceOk, ()> {
        self.add_boxed_device(Box::new(dev), port)
    }
}
// [-] INIT
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━



impl<'a> StdProcessor<'a> {
    #[inline]
    fn get_cur_dev(&mut self) -> Option<&mut Box<dyn Dev + 'a>> {
        self.devs.get_mut(&self.port_regs[self.reg_cur_pr])
    } 

    #[inline]
    fn result_error_dev(&self) -> ProcessorRunResult { 
        let port_reg = self.reg_cur_pr;
        ProcessorRunResult::ErrorDev{ port_reg, port: self.port_regs[port_reg] } 
    }

    #[inline]
    fn result_no_dev(&self) -> ProcessorRunResult { 
        let port_reg = self.reg_cur_pr;
        ProcessorRunResult::NoDev{ port_reg, port: self.port_regs[port_reg] } 
    }
    
    #[inline]
    fn result_inf_dev(&self) -> ProcessorRunResult { 
        let port_reg = self.reg_cur_pr;
        ProcessorRunResult::InfinityDev{ port_reg, port: self.port_regs[port_reg] } 
    }

    #[inline]
    fn get_main_reg(&self) -> u8 { self.main_regs[self.reg_cur_mr] }
    #[inline]
    fn set_main_reg(&mut self, value: u8) { self.main_regs[self.reg_cur_mr] = value; }

    pub fn run(&mut self) -> ProcessorRunResult {
        'cmd_loop: loop {
            // ━━━━ ━━━━ ━━━━ ━━━━ ━━━━ ━━━━ ━━━━ ━━━━
            // [+] GET CUR CMD
            let pr_com = self.port_regs[PR_COM];
            let com = self.devs.get_mut(&pr_com);
            if com.is_none() { return ProcessorRunResult::NoCom }
            let com = com.unwrap();

            if com.have_error() { return ProcessorRunResult::ErrorCom }
            if com.in_infinity_state() { return ProcessorRunResult::InfinityCom }
            if !com.test_can_read_byte() { return ProcessorRunResult::Ok }

            let cmd = com.read_byte();
            if com.have_error() { return ProcessorRunResult::ErrorCom }
            if com.in_infinity_state() { return ProcessorRunResult::InfinityCom }
            // [-] GET CUR CMD
            // ━━━━ ━━━━ ━━━━ ━━━━ ━━━━ ━━━━ ━━━━ ━━━━

            // ━━━━ ━━━━ ━━━━ ━━━━ ━━━━ ━━━━ ━━━━ ━━━━
            // [+] CMD PROCESSING

            if self.byte_await.is_some() {
                let byte = cmd;

                let already = self.byte_await.as_mut().unwrap();
                already.push(byte);

                if byte < MIN_BIG_BYTE { 
                    let se = std_se_decoding(already.iter());
                    if se.is_none() { return ProcessorRunResult::ErrorCmd }
                    
                    let se_value = se.unwrap();
                    if se_value > (PR_AMOUNT - 1) { return ProcessorRunResult::ErrorCmd }

                    self.reg_cur_pr = se_value;
                    self.byte_await = None;
                }

                continue 'cmd_loop
            }

            if let Some(rcn) = RegCmdNames::try_from_byte(cmd) {
                let mr_value = self.get_main_reg();
                match rcn {
                    RegCmdNames::Zero => { self.set_main_reg(0) }
                    RegCmdNames::And => { self.set_main_reg(mr_value & 0b_0000_000_1) }
                    RegCmdNames::Bnd => { self.set_main_reg(mr_value & 0b_1_000_0000) }
                    RegCmdNames::Inc => { self.set_main_reg(u8::overflowing_add(mr_value, 1).0) }
                    RegCmdNames::Dec => { self.set_main_reg(u8::overflowing_sub(mr_value, 1).0) }
                    RegCmdNames::LeftShift =>  { self.set_main_reg(mr_value << 1) }
                    RegCmdNames::RightShift => { self.set_main_reg(mr_value >> 1) }
                    RegCmdNames::TestZero => { self.set_main_reg(if mr_value == 0 { 0 } else { 1 })  }
                }

                continue 'cmd_loop
            }

            if let Some(scn) = StdCmdNames::is_start_byte(cmd) {
                match scn {
                    StdCmdNames::Pass => { }
                    
                    StdCmdNames::Test => {
                        let dev = self.get_cur_dev();
                        
                        let can_read = 
                        if let Some(dev) = dev {  dev.test_can_read_byte() } 
                        else { false }; // if no dev => we cant read from it
                        
                        self.set_main_reg(can_read as u8);
                    }
                    
                    StdCmdNames::Cur(_) => { self.byte_await = Some( vec![] ); }

                    StdCmdNames::Write => {
                        let wr_byte = self.get_main_reg(); 
                        if let Some(dev) = self.get_cur_dev() { dev.write_byte(wr_byte) } 
                    }

                    StdCmdNames::Read => {
                        let read_byte;

                        if let Some(dev) = self.get_cur_dev() {  
                            if dev.have_error() { return self.result_error_dev() }
                            if dev.in_infinity_state() { return self.result_inf_dev() }
                            read_byte = dev.read_byte();
                            if dev.have_error() { return self.result_error_dev() }
                            if dev.in_infinity_state() { return self.result_inf_dev() }
                        } 
                        else { return self.result_no_dev() }

                        self.set_main_reg(read_byte);
                    }

                    StdCmdNames::Set => {
                        let pr_cem = self.port_regs[PR_CEM];
                        let cem = self.devs.get_mut(&pr_cem);
                        if cem.is_none() { return ProcessorRunResult::NoCem }
                        let cem = cem.unwrap();
            
                        if cem.have_error() { return ProcessorRunResult::ErrorCem }
                        if cem.in_infinity_state() { return ProcessorRunResult::InfinityCem }
            
                        let mut se = vec![];
                        'se_read: loop {
                            let byte = cem.read_byte();
                            if cem.have_error() { return ProcessorRunResult::ErrorCem }
                            if cem.in_infinity_state() { return ProcessorRunResult::InfinityCem }

                            cem.write_byte(CellMemDevStartAction::NextCell as u8);
                            if cem.have_error() { return ProcessorRunResult::ErrorCem }
                            if cem.in_infinity_state() { return ProcessorRunResult::InfinityCem }

                            se.push(byte);
                            if byte < MIN_BIG_BYTE { break 'se_read }
                        }

                        let se_value = 
                            if let Some(x) = std_se_decoding(se.iter()) { x }
                            else { return ProcessorRunResult::ErrorCmd };
                        
                        if se_value > self.port_amount { return ProcessorRunResult::ErrorCmd }

                        self.port_regs[self.reg_cur_pr] = se_value;
                    }

                    StdCmdNames::SwapMainReg => { self.reg_cur_mr = (self.reg_cur_mr + 1) % MR_AMOUNT; }
                }

                continue 'cmd_loop
            }

            return ProcessorRunResult::UnknownCom{ cmd }
            // [-] CMD PROCESSING
            // ━━━━ ━━━━ ━━━━ ━━━━ ━━━━ ━━━━ ━━━━ ━━━━
        }
    }
}

pub enum AddDeviceOk{
    Ok,
    OldDevDisconected,
}

pub enum ProcessorRunResult {
    Ok,

    Infinity,
    InfinityCom,
    InfinityCem,
    InfinityDev{ port_reg:usize, port:usize },

    ErrorCmd,

    ErrorCom,
    ErrorCem,
    ErrorDev{ port_reg:usize, port:usize },
    
    NoDev{ port_reg:usize, port:usize },
    NoCom,
    NoCem,

    UnknownCom{ cmd: u8 },
}
