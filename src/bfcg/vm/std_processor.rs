use std::collections::HashMap;
use crate::bfcg::dev_emulators::dev::Dev;
use crate::bfcg::compiler::comand_compiler::{MIN_PORT_AMOUNT, MEM_CMD_PR, MEM_CELL_PR};

pub struct StdProcessor<'a> {
    // if need impl parallel work with the same dev by dif processor 
    // then need use Rc<RefCell<dyn Dev>>
    devs: HashMap<usize, Box<dyn Dev + 'a>>,
    port_amount: usize,

    main_reg: u8,
    port_regs: [usize; PR_AMOUNT],
}

const PR_AMOUNT:usize = MIN_PORT_AMOUNT + 1;
const PR_COM:usize = MEM_CMD_PR;
const PR_CEM:usize = MEM_CELL_PR;

// ----------------------------------------------------
// [+] INIT 
impl<'a> StdProcessor<'a> {
    pub fn new(port_amount: usize, pr_com: usize, pr_cem: usize) -> Self {
        let mut port_regs = [0; PR_AMOUNT];
        port_regs[PR_COM] = pr_com;
        port_regs[PR_CEM] = pr_cem;

        Self { 
            devs: HashMap::new(),
            port_amount,

            main_reg: 0,
            port_regs,
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
// ----------------------------------------------------

impl<'a> StdProcessor<'a> {
    
}

pub enum AddDeviceOk{
    Ok,
    OldDevDisconected,
}