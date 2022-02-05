use std::collections::HashMap;
use crate::bfcg::dev_emulators::dev::Dev;
use crate::bfcg::compiler::comand_compiler::{MIN_PORT_AMOUNT, MEM_CMD_PR, MEM_CELL_PR, RegCmdNames, StdCmdNames};

pub struct StdProcessor<'a> {
    // if need impl parallel work with the same dev by dif processor 
    // then need use Rc<RefCell<dyn Dev>>
    devs: HashMap<usize, Box<dyn Dev + 'a>>,
    port_amount: usize,

    byte_await: Option<ProcessorAwaitNextByte>,

    main_reg: u8,
    port_regs: [usize; PR_AMOUNT],
    reg_cur_pr: usize,
}

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

            main_reg: 0,
            port_regs,
            reg_cur_pr: 0,
        }
    }

    pub fn add_device<D: 'a + Dev>(&mut self, dev: D, port: usize) -> Result<AddDeviceOk, ()> {
        if port >= self.port_amount { return Err(()) }

        if let Some(_) = self.devs.insert(port, Box::new(dev)) { 
            Ok(AddDeviceOk::OldDevDisconected)
        } else {
            Ok(AddDeviceOk::Ok)
        }
    }
}
// [-] INIT
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const MODULE: i32 = 1 << 8; // 256

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
            if let Some(rcn) = RegCmdNames::try_from_byte(cmd) {
                match rcn {
                    RegCmdNames::And => { self.main_reg &= 0b_0000_0001 }
                    RegCmdNames::Bnd => { self.main_reg &= 0b_1000_0000 }
                    RegCmdNames::Inc => { self.main_reg = u8::overflowing_add(self.main_reg, 1).0 }
                    RegCmdNames::Dec => { self.main_reg = u8::overflowing_sub(self.main_reg, 1).0 }
                    RegCmdNames::LeftShift => { self.main_reg <<= 1 }
                    RegCmdNames::RightShift => { self.main_reg >>= 1 }
                    RegCmdNames::TestZero => { self.main_reg = if self.main_reg == 0 { 0 } else { 1 }  }
                }

                continue 'cmd_loop
            }

            if let Some(scn) = StdCmdNames::is_start_byte(cmd) {
                match scn {
                    StdCmdNames::Pass => {},
                    StdCmdNames::Test => {
                        let dev = self.get_cur_dev();
                        
                        let can_read = 
                        if let Some(dev) = dev {  dev.test_can_read_byte() } 
                        else { false }; // if no dev => we cant read from it
                        
                        self.main_reg = can_read as u8;
                    },
                    StdCmdNames::Cur(_) => {
                        self.byte_await = Some( ProcessorAwaitNextByte::CmdCur { already: vec![] } );
                    },

                    StdCmdNames::SetRegConst(_) => {
                        // TODO: CmdSetRegConst --> ZER
                        self.byte_await = Some( ProcessorAwaitNextByte::CmdSetRegConst );
                    },
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

    ErrorCom,
    ErrorDev{ port_reg:usize, port:usize },
    
    NoDev{ port_reg:usize, port:usize },
    NoCom,

    UnknownCom{cmd: u8},
}

enum ProcessorAwaitNextByte {
    CmdCur{ already: Vec<u8> },
    CmdSetRegConst,
}