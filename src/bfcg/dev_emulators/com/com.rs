use crate::{bfcg::{dev_emulators::{dev::Dev, dev_utilities::mem_dev::CmdMemDevStartAction}, general::se_fn::{MIN_BIG_BYTE, std_se_decoding}}, 
    dev_std_precheck_write_byte, dev_std_precheck_read_byte, 
    dev_std_realise_in_inf, dev_std_realise_have_error
};
use super::com_inner::ComInner;

// -------------------------------------------------
// [+] help struct 

struct StartedActionInfo{
    pub action_type: Option<CmdMemDevStartAction>,
    pub readed_cmd: bool, 
}

impl StartedActionInfo {
    pub fn new() -> Self { 
        Self { 
            action_type: None, 
            readed_cmd: false, 
        }
    }

    pub fn get_act(&self) -> Option<CmdMemDevStartAction> { self.action_type.clone() }

    pub fn can_read(&self) -> bool {
        !(self.readed_cmd && self.action_type.is_none())
    } 

    pub fn set_readed(&mut self) {
        if self.action_type.is_none() { panic!("[ALGO ERROR]") }
        self.readed_cmd = true;
    } 

    pub fn set_act_type(&mut self, act: CmdMemDevStartAction) {
        if self.action_type.is_some() { panic!("[ALGO ERROR]") }
        self.action_type = Some(act);
    }

    pub fn inform_read(&mut self) {
        if self.action_type.is_some() { self.set_readed() }
    }

    pub fn readed_cmd(&self) -> bool { self.readed_cmd }

    pub fn clear(&mut self) { 
        self.action_type = None;
        self.readed_cmd = false; 
    }
}

// [-] help struct 
// -------------------------------------------------


// -------------------------------------------------
// [+] DEV

pub struct DevCom {
    started_action: StartedActionInfo,

    inner: ComInner,
    error: bool,
    infinity: bool,
}

impl DevCom {
    pub fn new(mem_size: usize) -> Self {
        Self {
            started_action: StartedActionInfo::new(),

            inner: ComInner::new(mem_size),
            error: false,
            infinity: false,
        }
    }

    pub fn move_to_start(&mut self) { self.inner.move_to_start() }
    pub fn set_mem(&mut self, mem: Vec<u8>) { self.inner.set_mem(mem); }
    pub fn init<'a, Iter: Iterator<Item = &'a u8>>(&mut self, iter: Iter) { self.inner.init(iter); }

    pub fn inner_read_byte(&mut self) -> u8 {
        if let Some(x) = self.inner.get_cur_cell() { 
            self.inner.move_forward();
            x
        } else {
            self.error = true;
            DEFAULT
        }     
    }
}

const DEFAULT:u8 = 0x00;

impl Dev for DevCom {
    fn read_byte(&mut self) -> u8 {
        if !self.started_action.can_read() { self.error = true; }
        self.started_action.inform_read();
        
        dev_std_precheck_read_byte!(self, DEFAULT);
        
        self.inner_read_byte()
    }

    fn write_byte(&mut self, byte: u8) {
        dev_std_precheck_write_byte!(self);

        if let Some(act) = self.started_action.get_act() {
            if !self.started_action.readed_cmd() { self.error = true; return } // how are you get here?

            // now read jump len and decide do jump or not (by check `byte`) 
            let mut vec = vec![];
            loop {
                let x = self.inner_read_byte();
                vec.push(x);
                if x < MIN_BIG_BYTE { break; }
            }
            if self.error { return }
            let se_value = std_se_decoding(vec.iter());
            let jmp_value = if let Some(x) = se_value { x } else { self.error = true; return };
            
            let jmp_len_forward = jmp_value - vec.len();
            let jmp_len_backward = jmp_value + vec.len();

            match act {
                CmdMemDevStartAction::JumpForward => {
                    if byte == 0 { self.inner.jump_forward(jmp_len_forward) }
                }
                CmdMemDevStartAction::JumpBackward => {
                    if byte != 0 { self.inner.jump_forward(jmp_len_backward) }
                }
            }

            self.started_action.clear();
        } else {
            let act = CmdMemDevStartAction::try_from_byte(byte);
            if act.is_none() { self.error = true; return }
            self.started_action.set_act_type(act.unwrap());
        }
    }

    fn test_can_read_byte(&self) -> bool {
        if self.have_error() | self.in_infinity_state() { return false }
        return self.inner.stay_on_end()
    }

    dev_std_realise_in_inf!();
    dev_std_realise_have_error!();
}

// [-] DEV
// -------------------------------------------------