use std::u8;

use crate::{bfcg::dev_emulators::{dev::Dev, dev_utilities::mem_dev::CellMemDevStartAction}, dev_std_precheck_read_byte, dev_std_precheck_write_byte};

use super::cem_inner::{CemInner};



pub struct DevStdCem{
    cem_inner: CemInner, 

    infinity: bool,
    error: bool,

    start_byte_seq: Option<CellMemDevStartAction>, // only None | Some({Get, Set}CellValue)
}

impl DevStdCem {
    pub fn new(mm_size: usize, am_size: usize) -> Self {
        Self {
            cem_inner: CemInner::new(mm_size, am_size),
            
            infinity: false,
            error:false,
            
            start_byte_seq: None,
        }
    }

    pub fn memory_print(&self) -> String {
        let mut ret = String::new();

        ret += "OM: ";
        ret += &self.cem_inner.print_om(0);
        
        ret += "\nMM: ";
        ret += &self.cem_inner.print_mm(0);
        
        ret += "\nAM: ";
        ret += &self.cem_inner.print_am(0);

        ret
    }
}


// -----------------------------------------------
// [+] COND HELPER:
impl DevStdCem {
    const fn write_byte_cond(&self, x: u8, action: CellMemDevStartAction) -> bool {
        if let None = self.start_byte_seq { x == action as u8 }
        else { false }
    }
}
// [-] COND HELPER
// -----------------------------------------------

const DEFAULT:u8 = 0x00;
const ALGO_ERROR:&'static str = "[ALGO ERROR]";

impl Dev for DevStdCem {
    fn read_byte(&mut self) -> u8 {
        dev_std_precheck_read_byte!(self, DEFAULT);

        let value = self.cem_inner.get_value();

        if let Some(x) = value { return x }
        else {
            if self.have_error() { return DEFAULT }
            else { panic!("{}", ALGO_ERROR) }
        }
    }

    fn write_byte(&mut self, byte: u8) { 
        dev_std_precheck_write_byte!(self);

        match byte {
            // GET:
            x if self.write_byte_cond(x, CellMemDevStartAction::GetCellValue) => {
                self.start_byte_seq = Some(CellMemDevStartAction::GetCellValue);
            }

            // SET:
            x if self.write_byte_cond(x, CellMemDevStartAction::SetCellValue) => {
                self.start_byte_seq = Some(CellMemDevStartAction::SetCellValue);
            }
            x if matches!(self.start_byte_seq, Some(CellMemDevStartAction::SetCellValue)) => {
                self.cem_inner.set_value(x);
                self.start_byte_seq = None;
            }

            // PREV
            x if self.write_byte_cond(x, CellMemDevStartAction::PrevCell) => {
                self.cem_inner.prev_cell();
            }

            // NEXT
            x if self.write_byte_cond(x, CellMemDevStartAction::NextCell) => {
                self.cem_inner.next_cell();
            }

            // CREATE
            x if self.write_byte_cond(x, CellMemDevStartAction::CreateCell) => {
                self.cem_inner.create_cell();
            }

            // DELETE
            x if self.write_byte_cond(x, CellMemDevStartAction::CreateCell) => {
                self.cem_inner.delete_cell();
            }

            _ => { self.error = true; }
        }
    }

    fn test_can_read_byte(&self) -> bool {
        if self.have_error() | self.in_infinity_state() { return false }

        if let Some(CellMemDevStartAction::GetCellValue) = self.start_byte_seq { true }
        else { false }
    }

    fn have_error(&self) -> bool { self.error || self.cem_inner.error() }
    fn in_infinity_state(&self) -> bool { self.infinity }
}