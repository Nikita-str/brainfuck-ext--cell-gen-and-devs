use crate::{bfcg::dev_emulators::dev::Dev, 
    dev_std_precheck_write_byte, dev_std_precheck_read_byte, 
    dev_std_realise_in_inf, dev_std_realise_have_error, dev_ctor_no_param_impl
};
use super::console_inner::ConsoleInner;

pub struct DevConsoleAscii{
    inner: ConsoleInner,
    error: bool,
    infinity: bool,
}

impl DevConsoleAscii {
    pub fn new() -> Self{
        Self{ 
            inner: ConsoleInner::new(),
            error: false,
            infinity: false,
        }
    }

    fn ok_char(c: char) -> bool { c.is_ascii_graphic() || c.is_ascii_whitespace() }
    fn ok_byte(x: u8) -> bool { x.is_ascii_graphic() || x.is_ascii_whitespace() }
}

const DEFAULT:u8 = 0x00;

impl Dev for DevConsoleAscii {
    fn read_byte(&mut self) -> u8 {
        dev_std_precheck_read_byte!(self, DEFAULT);

        if let Some(c) = self.inner.read_char() {
            if Self::ok_char(c) { return c as u8 }
        }
            
        self.error = true; 
        return DEFAULT
    }

    fn write_byte(&mut self, byte: u8) {
        dev_std_precheck_write_byte!(self);

        if Self::ok_byte(byte) { self.inner.write_char(byte as char) }
        else { self.error = true; }
    }

    fn test_can_read_byte(&self) -> bool {
        if self.have_error() | self.in_infinity_state() { return false }
        return true
    }

    dev_std_realise_in_inf!();
    dev_std_realise_have_error!();
}
impl crate::bfcg::dev_emulators::dev::ToDevComInit for DevConsoleAscii {}


dev_ctor_no_param_impl!(DevConsoleAscii);