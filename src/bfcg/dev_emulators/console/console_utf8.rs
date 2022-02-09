use std::collections::LinkedList;

use crate::{bfcg::dev_emulators::dev::Dev, 
    dev_std_precheck_write_byte, dev_std_precheck_read_byte, 
    dev_std_realise_in_inf, dev_std_realise_have_error, dev_ctor_one_param_impl
};
use super::console_inner::{ConsoleInner, PrivateConsoleNeedWrite, DEFAULT_NEED_WRITE_STATE};

pub struct DevConsoleUtf8 {
    readed_buffer: LinkedList<u8>,

    writed_buffer: Vec<u8>,
    write_len: usize,
    
    inner: ConsoleInner,
    error: bool,
    infinity: bool,
}

impl DevConsoleUtf8{
    fn new(need_write_state: PrivateConsoleNeedWrite) -> Self {
        Self {
            readed_buffer: LinkedList::new(),

            writed_buffer: Vec::new(),
            write_len: 0,

            inner: ConsoleInner::new(need_write_state),
            error: false,
            infinity: false,
        }
    }
}

const DEFAULT:u8 = 0x00;

impl Dev for DevConsoleUtf8 {

    /// each character return as seq: `| N = len char of UTF8 | X[0] | .. | X[N] |`; so len of seq = N + 1
    fn read_byte(&mut self) -> u8 {
        dev_std_precheck_read_byte!(self, DEFAULT);
        
        if self.write_len != 0 { self.infinity = true; return DEFAULT }
        if self.readed_buffer.len() > 0 { return self.readed_buffer.pop_front().unwrap()  }

        if let Some(c) = self.inner.read_char() {
            let len = char::len_utf8(c);
            let s = c.to_string();
            let bytes = s.as_bytes(); 
            if bytes.len() != len { panic!("[ALGO ERROR]") }
            for x in bytes { self.readed_buffer.push_back(*x) }
            return len as u8;
        } else {
            if !self.have_error() { panic!("[ALGO ERROR]") }
            return DEFAULT
        }
    }

    fn write_byte(&mut self, byte: u8) {
        dev_std_precheck_write_byte!(self);

        if self.readed_buffer.len() > 0 { self.error = true; return }
        if self.write_len == 0 {
            if byte == 0 || byte > 4 { self.error = true; return } // potential additional cmds can start with 0 U [5; 0xFF]
            self.write_len = byte as usize;
        } else {
            self.writed_buffer.push(byte);
            if self.writed_buffer.len() == self.write_len { // => finnaly write char in console
                let mut u32_value:u32 = 0;
                for x in &self.writed_buffer { u32_value = (u32_value << 8) | (*x as u32); } // get utf-8 u32 value
                if let Some(c) = char::from_u32(u32_value) {
                    self.inner.write_char(c);
                    self.writed_buffer.clear();
                    self.write_len = 0;
                } else {
                    self.error = true;
                    return
                }
            }
        }
    }

    fn test_can_read_byte(&self) -> bool {
        if self.have_error() | self.in_infinity_state() { return false }
        return true
    }

    dev_std_realise_in_inf!();
    dev_std_realise_have_error!();
}
impl crate::bfcg::dev_emulators::dev::ToDevComInit for DevConsoleUtf8 {}

dev_ctor_one_param_impl!(DevConsoleUtf8, "print-state", DEFAULT_NEED_WRITE_STATE);
