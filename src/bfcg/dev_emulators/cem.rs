
// MAYBE: we can use (consecutive-xx-amount == 0xF) instead overflow-xx-flag


// main, additional & order can be on the same memory device, 
//       just in different pos, like this:
//               |   main   | additional | order |
//
/// this device is sequential access memory device  
struct CemInner{
    main_mem: Vec<u8>,
    mm_size: usize,
    mm_pos: usize, // {*1} : ptr on cur cell is enough for real CEM device

    additional_mem: Vec<u8>,
    am_size: usize,
    am_pos: Option<usize>, // check {*1}

    /// each u8 has next struct:
    /// ```
    /// |     1 bit      |        3 bit        |     1 bit      |        3 bit        | 
    /// |overflow-mm-flag|consecutive-mm-amount|overflow-am-flag|consecutive-am-amount|
    /// ```
    /// overflow-xx-flag stay:
    /// * 1 if consecutive-xx-amount not fit in 3 bit & real consecutive amount is `Sum(cons-xx-amount, while overflow-xx-flag == 1) + next cons-xx-amount (of == 0)`
    /// * 0 if consecutive-xx-amount fit in 3 bit or if it end of previous unfited seq
    order_mem: Vec<u8>,
    om_size: usize,

    om_mm_end_pos: usize, // {*1} (ptr on end used cell) 
    om_mm_pos: usize, // main_memory pos in om, check {*1} but we need 3 ptr
    om_am_pos: usize, // addi_memory pos in om, check {*1} but we need 3 ptr
    reg_cur_am: bool, // if 0 => mm else am

    reg_con_amount: u8, // u3 is enough 

    /// for emulate invalid operation  
    error: bool,
}

const MM_OF_SHIFT: usize = 7;
const AM_OF_SHIFT: usize = 3;
const MM_AMOUNT_SHIFT: usize = 4;
const AM_AMOUNT_SHIFT: usize = 0;

const AMOUNT_MAX:u8 = 7;
const MM_AMOUNT_MASK: u8 = AMOUNT_MAX << MM_AMOUNT_SHIFT;

// --------------------------------------------
// [+] OM fn
impl CemInner{
    /// ## panic 
    /// * if mm_amount > 7 || am_amount > 7
    fn create_om_cell(mm_of: bool, mm_amount: u8, am_of: bool, am_amount: u8) -> u8 {
        if (mm_amount >= (1 << 3)) || (am_amount >= (1 << 3)) { panic!("too big value: mm_amount = {}, am_amount = {}", mm_amount, am_amount) }
        
        let mut om_cell = if mm_of { 1u8 << MM_OF_SHIFT } else { 0 };
        om_cell |= if am_of { 1u8 << AM_OF_SHIFT } else { 0 };
        om_cell |= mm_amount << MM_AMOUNT_SHIFT;
        om_cell |= am_amount << AM_AMOUNT_SHIFT;

        om_cell
    }

    fn om_inc_mm_pos(&mut self) {
        self.reg_cur_am = false;
        self.om_mm_pos += 1;
        if self.om_mm_pos > self.om_am_pos { self.order_mem.push(0x00); } // init cell, in real dev it already 0x00 
        if self.om_mm_pos > self.om_mm_end_pos { self.om_mm_end_pos = self.om_mm_pos; }
    }

    fn om_inc_mm_con(&mut self) {
        let mut om_x = self.order_mem[self.om_mm_pos];
        let mut x = (om_x >> MM_AMOUNT_SHIFT) & AMOUNT_MAX;   
        if x == AMOUNT_MAX { 
            self.order_mem[self.om_mm_pos] = om_x | (1 << MM_OF_SHIFT);
            self.om_inc_mm_pos();
            om_x = 0x00;
            x = 0;
        } 
        let x = x + 1;
        self.order_mem[self.om_mm_pos] = (om_x & !MM_AMOUNT_MASK) | (x << MM_AMOUNT_SHIFT);
        self.reg_con_amount = x;
    }
}
// [-] OM fn
// --------------------------------------------

impl CemInner{
    pub fn new(mm_size: usize, am_size: usize) -> Self{
        // if do mm_size it can take too many memory, I dont think its really necessary.
        // for more realistic: yes we must do capasity mm_size & init it by zeros. 
        // but it emulator, so: no
        // in the same time, inner structure is real, it must work if we do it real hardware 
        // so only unrealistic thing is the use of dynamic memory 
        let mut mm = Vec::with_capacity(std::cmp::min(1 << 8, mm_size)); 
        mm.push(0x00);

        let am = Vec::with_capacity(std::cmp::min(1 << 8, am_size));

        // om_size can be counted more precisely, if do it in right way then it can economy ~7/8 
        //                                        in case when {mm_size >> am_size} or {mm_size << am_size} 
        let om_size = std::cmp::max(mm_size, am_size); 
        let mut om = Vec::with_capacity(std::cmp::min(1 << 8, om_size));
        om.push(Self::create_om_cell(false, 1, false, 0));

        Self {
            main_mem: mm,
            mm_size,
            mm_pos: 0,

            additional_mem: am,
            am_size,
            am_pos: None, // eq -1 - special cell located before any real used cell

            order_mem: om,
            om_size,

            om_mm_end_pos: 0,
            om_mm_pos: 0,
            om_am_pos: 0,
            reg_cur_am: false, // first mem-cell always is mm even in run-time

            reg_con_amount: 1,

            error: (mm_size == 0),
        }
    }

    fn cur_con_end(&self) -> bool {
        let con_amount_real = 
            if self.reg_cur_am {
                (self.order_mem[self.om_am_pos] >> AM_AMOUNT_SHIFT) & AMOUNT_MAX
            } else {
                (self.order_mem[self.om_mm_pos] >> MM_AMOUNT_SHIFT) & AMOUNT_MAX
            };
        if con_amount_real < self.reg_con_amount { panic!("[ALGO ERROR] amount_real = {} < reg_amount = {}", con_amount_real, self.reg_con_amount) } 
        con_amount_real == self.reg_con_amount
    }

    fn stay_on_the_end(&self) -> bool {
        let am_can_be_end = (self.am_pos.is_none() && self.am_size == 0)
            || (self.am_pos.is_some() && self.am_pos.unwrap() == (self.am_size - 1)) 
            || (self.order_mem.len() == self.om_am_pos + 1)
            || (((self.order_mem[self.om_am_pos + 1] >> AM_AMOUNT_SHIFT) & AMOUNT_MAX) == 0);
        
        let mm_can_be_end = (self.mm_pos == (self.mm_size - 1)) 
            || (self.order_mem.len() == self.om_mm_pos + 1)
            || (((self.order_mem[self.om_mm_pos + 1] >> MM_AMOUNT_SHIFT) & AMOUNT_MAX) == 0);

        let con_end = self.cur_con_end();

        am_can_be_end && mm_can_be_end && con_end
    }

    fn stay_on_the_start(&self) -> bool {
        if self.reg_cur_am { false } 
        else { self.reg_con_amount == 1 && self.mm_pos == 0 }
    }

    fn cur_con_overflow(&self) -> bool {
        if self.reg_cur_am {
            ((self.order_mem[self.om_am_pos] >> AM_OF_SHIFT) & 1) == 1
        } else {
            ((self.order_mem[self.om_mm_pos] >> MM_OF_SHIFT) & 1) == 1
        }
    }

    fn cur_con_prev_overflow(&self) -> bool {
        if self.reg_cur_am {
            let om_am_pos = self.om_am_pos;
            (om_am_pos > 0) && ((self.order_mem[om_am_pos - 1] >> AM_OF_SHIFT) & 1 == 1)
        } else {
            let om_mm_pos = self.om_mm_pos;
            (om_mm_pos > 0) && ((self.order_mem[om_mm_pos - 1] >> MM_OF_SHIFT) & 1 == 1)
        }
    }
    
    /// [NOT NECESSARY]
    /// 
    /// auto pseudo-delete mm 
    /// 
    /// USE ONLY IN PREV!
    /// ## Ret value
    /// * true if pseudo-deleted
    /// * false else 
    fn try_pseudo_mm_del(&mut self, on_con_end: bool) -> bool {
        if self.error { return false }

        if on_con_end && !self.reg_cur_am && (self.om_mm_pos == self.om_mm_end_pos) && (self.get_value().unwrap() == 0x00) {
            // we just dec counter, so it take ~0 overhead & easier than 
            //                      save amount of already pushed for not push more than need: 
            self.main_mem.pop(); 

            if self.reg_con_amount == 0 {
                self.order_mem[self.om_mm_pos] = self.order_mem[self.om_mm_pos] & !MM_AMOUNT_MASK & !(1 << MM_OF_SHIFT);
                if self.order_mem[self.om_mm_pos] == 0x00 { self.order_mem.pop(); }
            } else {
                self.order_mem[self.om_mm_pos] = (self.order_mem[self.om_mm_pos] & !MM_AMOUNT_MASK) | (self.reg_con_amount << MM_AMOUNT_SHIFT);
            }
            return true
        }

        false
    }

    pub fn error(&self) -> bool { self.error }

    pub fn next_cell(&mut self) {
        if self.error { return }

        if self.stay_on_the_end() {
            if self.mm_pos == (self.mm_size - 1) { self.error = true; return }
            self.main_mem.push(0x00);
            self.mm_pos += 1;
            
            if self.reg_cur_am { self.om_inc_mm_pos(); }
            self.om_inc_mm_con(); 
        } else {
            if self.cur_con_end() {
                self.reg_con_amount = 1;
                if !self.cur_con_overflow() { self.reg_cur_am = !self.reg_cur_am; }
                if self.reg_cur_am {
                    self.am_pos = 
                        if let Some(x) = self.am_pos { Some(x + 1) } 
                        else { panic!("[ALGO ERROR]: cause cur is overflow => next already exist") }; 
                    self.om_am_pos += 1;
                } else {
                    self.mm_pos += 1;
                    self.om_mm_pos += 1;
                }
            } else {
                self.reg_con_amount += 1;
                if self.reg_cur_am { 
                    self.am_pos = 
                        if let Some(x) = self.am_pos { Some(x + 1) } 
                        else { panic!("[ALGO ERROR]: cause cur is not con_end => next already exist") }; 
                }
                else { self.mm_pos += 1; }
            }
        }
    }

    pub fn prev_cell(&mut self) {
        if self.error { return }

        let on_con_end = self.cur_con_end();

        if self.reg_con_amount == 1 {
            if self.stay_on_the_start() { self.error = true; return }

            self.reg_con_amount = 0;
            // [NOT NECESSARY]
            let mut need_null_mm_of = self.try_pseudo_mm_del(on_con_end);

            if !self.cur_con_prev_overflow() { 
                self.reg_cur_am = !self.reg_cur_am;
                need_null_mm_of = false;
            }

            if self.reg_cur_am {
                self.am_pos = Some(self.am_pos.unwrap() - 1);
                self.om_am_pos -= 1;
                self.reg_con_amount = (self.order_mem[self.om_am_pos] >> AM_AMOUNT_SHIFT) & AMOUNT_MAX;
            } else {
                self.mm_pos -= 1;
                self.om_mm_pos -= 1;
                self.reg_con_amount = (self.order_mem[self.om_mm_pos] >> MM_AMOUNT_SHIFT) & AMOUNT_MAX;
                // if need nullify MM OF-flag:
                if need_null_mm_of { self.order_mem[self.om_mm_pos]  = self.order_mem[self.om_mm_pos] & !(1 << MM_OF_SHIFT); }
            }
        } else {
            self.reg_con_amount -= 1;
            if self.reg_cur_am { 
                self.am_pos = 
                    if let Some(x) = self.am_pos { Some(x - 1) } 
                    else { panic!("[ALGO ERROR]: cause cur is not con_start => prev exist") }; 
            } else {
                // [NOT NECESSARY]
                self.try_pseudo_mm_del(on_con_end);

                self.mm_pos -= 1;
            }
        }
    }

    pub fn set_value(&mut self, value: u8) {
        if self.error { return }
        if self.reg_cur_am { self.additional_mem[self.am_pos.unwrap()] = value; }
        else { self.main_mem[self.mm_pos] = value; }
    }

    pub fn get_value(&mut self) -> Option<u8> {
        if self.error { return None }
        if self.reg_cur_am { Some(self.additional_mem[self.am_pos.unwrap()]) }
        else { Some(self.main_mem[self.mm_pos]) }
    }
}

// ---------------------------------------------------------
// [+] PRINT
impl CemInner {
    pub fn print_om(&self, new_line_each: usize) -> String {
        let mut ret = String::new();
        ret.push('|');
        for (ind, x) in (&self.order_mem).into_iter().enumerate() {
            ret += &format!(
                "{}{}{}{}|", 
                if (x >> MM_OF_SHIFT) & 1 == 1 { 'T' } else {'F'},
                (x >> MM_AMOUNT_SHIFT) & AMOUNT_MAX, 
                if (x >> AM_OF_SHIFT) & 1 == 1 { 'T' } else {'F'},
                (x >> AM_AMOUNT_SHIFT) & AMOUNT_MAX, 
            );
            if (new_line_each > 0) && ind != 0 && (ind % new_line_each == 0) { ret += "\n"; } 
        }
        ret += "F0F0|...";
        ret
    } 

    pub fn print_mm(&self, new_line_each: usize) -> String {
        let mut ret = String::new();
        ret.push('|');
        for (ind, x) in (&self.main_mem).into_iter().enumerate() {
            ret += &format!("{:02X}|", x);
            if (new_line_each > 0) && ind != 0 && (ind % new_line_each == 0) { ret += "\n"; } 
        }
        ret += "00|...";
        ret
    }
    
    pub fn print_am(&self, new_line_each: usize) -> String {
        let mut ret = String::new();
        ret.push('|');
        for (ind, x) in (&self.additional_mem).into_iter().enumerate() {
            ret += &format!("{:02X}|", x);
            if (new_line_each > 0) && ind != 0 && (ind % new_line_each == 0) { ret += "\n"; } 
        }
        ret += "00|...";
        ret
    }
}
// [-] PRINT
// ---------------------------------------------------------



// ---------------------------------------------------------
// TEST

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn test_cem_01(){
        let mut cem = CemInner::new(1024, 1024);
        cem.next_cell();
        cem.set_value(2);
        cem.next_cell();
        cem.next_cell();
        assert_eq!(&cem.print_om(0), "|F4F0|F0F0|...");
        cem.next_cell();
        assert_eq!(&cem.print_om(0), "|F5F0|F0F0|...");
        cem.prev_cell();
        assert_eq!(&cem.print_om(0), "|F4F0|F0F0|...");
        cem.set_value(3);
        cem.next_cell();
        cem.next_cell();
        cem.next_cell();
        cem.set_value(4);
        assert_eq!(&cem.print_om(0), "|F7F0|F0F0|...");
        cem.next_cell();
        cem.next_cell();
        assert_eq!(&cem.print_om(0), "|T7F0|F2F0|F0F0|...");
        assert_eq!(&cem.print_mm(0), "|00|02|00|03|00|00|04|00|00|00|...");
        cem.prev_cell();
        cem.prev_cell();
        assert_eq!(&cem.print_om(0), "|F7F0|F0F0|...");
        cem.prev_cell();
        cem.prev_cell();
        assert_eq!(&cem.print_om(0), "|F7F0|F0F0|...");
        cem.set_value(1);
        assert_eq!(&cem.print_mm(0), "|00|02|00|03|01|00|04|00|...");
        assert_eq!(cem.error(), false);

        println!("OM: {}", cem.print_om(0));
        println!("MM: {}", cem.print_mm(0));
        println!("AM: {}", cem.print_am(0));
    }
}