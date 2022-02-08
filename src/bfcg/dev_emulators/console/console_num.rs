use crate::{bfcg::dev_emulators::dev::Dev, 
    dev_std_precheck_write_byte, dev_std_precheck_read_byte, 
    dev_std_realise_in_inf, dev_std_realise_have_error, dev_ctor_one_param_impl
};
use super::console_inner::ConsoleInner;


pub struct DevConsoleNum{
    /// how frequently write '\n'
    new_line_freq: usize,
    /// how much writed on this line
    writed_on_cur_line: usize,

    inner: ConsoleInner,
    error: bool,
    infinity: bool,
}


impl DevConsoleNum{
    pub fn new(new_line_freq: usize) -> Self {
        Self {
            new_line_freq,
            writed_on_cur_line: 0,

            inner: ConsoleInner::new(),
            error: false,
            infinity: false,
        }
    }
}


const DEFAULT:u8 = 0x00;
const MAX_BYTE_LEN:i32 = 3;  // "255" is max byte and it char-len is 3

impl Dev for DevConsoleNum {
    fn read_byte(&mut self) -> u8 {
        dev_std_precheck_read_byte!(self, DEFAULT);

        let mut len = 0;
        let mut num:u32 = 0;
        let mut was_num = false; 
        loop {
            if let Some(c) = self.inner.read_char() {
                let space = c.is_ascii_whitespace();
                if !was_num && space { continue }
                if was_num && space {  
                    if num <= u8::MAX as u32 { return num as u8 }
                    else { self.error = true; return DEFAULT }
                }

                if !c.is_ascii_digit() { self.error = true; return DEFAULT }
                was_num = true;
                len += 1;
                if len > MAX_BYTE_LEN { self.error = true; return DEFAULT }

                let x = c.to_digit( 10).unwrap();
                num = (num * 10) + x; 
            } else {
                self.error = true;
                return DEFAULT
            }
        }
    }

    fn write_byte(&mut self, byte: u8) {
        dev_std_precheck_write_byte!(self);

        for x in byte.to_string().chars() { self.inner.write_char(x); }
        self.inner.write_char(' ');

        if self.new_line_freq != 0 {
            self.writed_on_cur_line += 1;
            if self.writed_on_cur_line == self.new_line_freq {
                self.inner.write_char('\n');
                self.writed_on_cur_line = 0;
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
impl crate::bfcg::dev_emulators::dev::ToDevComInit for DevConsoleNum {}


const DEFAULT_NEW_LINE_FREQ: usize = 10;
dev_ctor_one_param_impl!(DevConsoleNum, "new-line", DEFAULT_NEW_LINE_FREQ);