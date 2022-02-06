use crate::{bfcg::dev_emulators::{dev::Dev, dev_utilities::win_dev::WinDevStartAction}, dev_std_precheck_write_byte, dev_std_precheck_read_byte};

use super::{Win, win::SpecialWin};

enum DevWinAwaitAct{
    Get,
    Set,
}

pub struct DevWin {
    inner: Win,
    buffer: Vec<u8>,

    pos_x: usize, 
    pos_y: usize, 
    /// RGBA : 0..4 
    pos_pixel: usize, 

    awaited_action: Option<DevWinAwaitAct>,

    error: bool,
    infinity: bool,
}

const PIXEL_BYTE: usize = 4; // RGBA

impl DevWin { 
    pub fn new(win: &mut SpecialWin) -> Self { 
        let (inner, buffer) = win.create_dev_helper();
        Self {
            inner,
            buffer,

            pos_x: 0,
            pos_y: 0,
            pos_pixel: 0,

            awaited_action: None,

            error: false,
            infinity: false,
        } 
    } 

    fn width(&self) -> usize { self.inner.get_width() as usize }
    fn height(&self) -> usize { self.inner.get_height() as usize }

    fn inc_x(&mut self) { self.pos_x = (self.pos_x + 1) % self.width() }
    fn inc_y(&mut self) { self.pos_y = (self.pos_y + 1) % self.height() }

    fn dec_x(&mut self) { 
        if self.pos_x == 0 { self.pos_x = self.width() - 1 } 
        else { self.pos_x -= 1 }
    }
    fn dec_y(&mut self) { 
        if self.pos_y == 0 { self.pos_y = self.height() - 1 } 
        else { self.pos_y -= 1 }
    }

    fn clear_pos(&mut self) { 
        self.pos_x = 0;
        self.pos_y = 0;
        self.pos_pixel = 0;
     }
    fn get_buffer_pos(&self) -> usize { self.pos_pixel + self.pos_x + self.width() * self.pos_y }

    fn set_byte(&mut self, byte: u8) { 
        let buffer_pos = self.get_buffer_pos();
        self.buffer[buffer_pos] = byte; 
    }
    fn get_byte(&mut self) -> u8 { self.buffer[self.get_buffer_pos()] }

    fn next_pixel_pos(&mut self) { self.pos_pixel %= PIXEL_BYTE; }
}


// -------------------------------------------------
// [+] DEV

const DEFAULT: u8 = 0xA4;

impl Dev for DevWin {
    fn read_byte(&mut self) -> u8 {
        dev_std_precheck_read_byte!(self, DEFAULT);

        // self.await checked in test_can_read_byte, that checked in std_precheck
        
        let ret = self.get_byte();
        self.next_pixel_pos();
        self.awaited_action = None;

        ret
    }

    fn write_byte(&mut self, byte: u8) {
        dev_std_precheck_write_byte!(self);
        
        match self.awaited_action {
            None => {}
            Some(DevWinAwaitAct::Get) => { self.error = true; return }
            Some(DevWinAwaitAct::Set) => { 
                self.set_byte(byte);
                self.next_pixel_pos();
                self.awaited_action = None;
                return
            }
        }

        match byte {
            x if x == WinDevStartAction::DecCoordX as u8 => { self.dec_x() }
            x if x == WinDevStartAction::DecCoordY as u8 => { self.dec_y() }
            x if x == WinDevStartAction::IncCoordX as u8 => { self.inc_x() }
            x if x == WinDevStartAction::IncCoordY as u8 => { self.inc_y() }

            x if x == WinDevStartAction::RedrawWin as u8 => { 
                if !self.inner.start_draw_frame() { self.error = true; return }
                self.inner.draw_from_buffer(&self.buffer);
                if !self.inner.end_draw_frame() { self.error = true; return }
                self.clear_pos();
            }

            x if x == WinDevStartAction::SetWinValue as u8 => {
                self.awaited_action = Some(DevWinAwaitAct::Set)
            }
            
            x if x == WinDevStartAction::GetWinValue as u8 => {
                self.awaited_action = Some(DevWinAwaitAct::Get)
            }

            _ => { self.error = true }
        }
    }

    fn test_can_read_byte(&self) -> bool { 
        if self.have_error() | self.in_infinity_state() { false }
        else { matches!(self.awaited_action, Some(DevWinAwaitAct::Get)) }
    }

    fn have_error(&self) -> bool { self.error }
    fn in_infinity_state(&self) -> bool { self.infinity }
}

// [-] DEV
// -------------------------------------------------
