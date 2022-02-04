// оставь надежду всяк сюда входящий
// code there is hard for understanding 
// sorry


// MAYBE: we can use (consecutive-xx-amount == 0xF) instead overflow-xx-flag


// main, additional & order can be on the same memory device, 
//       just in different pos, like this:
//               |   main   | additional | order |
//
/// this device is sequential access memory device  
pub(in super)
struct CemInner{
    /// \[MEM\]
    main_mem: Vec<u8>,
    /// \[HARDWARE INFO\]
    mm_size: usize,
    /// \[POINTER\]
    mm_pos: usize, // {*1} : ptr on cur cell is enough for real CEM device

    /// \[MEM\]
    additional_mem: Vec<u8>,
    /// \[HARDWARE INFO\]
    am_size: usize,
    /// \[POINTER\] None is special pos or special bit
    am_pos: Option<usize>, // check {*1}
    /// \[POINTER\] None is special pos or special bit
    am_last_clear_pos: Option<usize>, // check {*1}
    /// \[TRIGGER\] setted in false when am_pos < am_last_clear_pos
    trig_am_can_create: bool, 

    /// \[MEM\]
    /// each u8 has next struct:
    /// ```
    /// |     1 bit      |        3 bit        |     1 bit      |        3 bit        | 
    /// |overflow-mm-flag|consecutive-mm-amount|overflow-am-flag|consecutive-am-amount|
    /// ```
    /// overflow-xx-flag stay:
    /// * 1 if consecutive-xx-amount not fit in 3 bit & real consecutive amount is `Sum(cons-xx-amount, while overflow-xx-flag == 1) + next cons-xx-amount (of == 0)`
    /// * 0 if consecutive-xx-amount fit in 3 bit or if it end of previous unfited seq
    order_mem: Vec<u8>,
    #[allow(unused)]
    /// \[HARDWARE INFO\]
    om_size: usize,
    /// \[POINTER\]
    om_mm_end_pos: usize, // {*1} (ptr on end used cell) 
    /// \[POINTER\]
    om_mm_pos: usize, // main_memory pos in om, check {*1} but we need 3 ptr
    /// \[POINTER\]
    om_am_end_pos: usize, // {*1} (ptr on end used cell) 
    /// \[POINTER\]
    om_am_pos: usize, // addi_memory pos in om, check {*1} but we need 3 ptr
    /// \[TRIGGER\]
    trig_cur_am: bool, // if 0 => mm else am

    /// \[REG\] \[COUNTER\]
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
const AM_AMOUNT_MASK: u8 = AMOUNT_MAX << AM_AMOUNT_SHIFT;


// --------------------------------------------
// [+] inline fn

#[derive(PartialEq, Eq, Clone, Copy)]
enum OrdMemType{
    Addi,
    Main,
}


macro_rules! algo_error { ( ) => {  panic!("[ALGO ERROR]")  }; }

impl CemInner{
    #[inline(always)]
    fn on_of_flag(&mut self, pos: usize, om_type: OrdMemType) {
        let shift = 
        if om_type == OrdMemType::Addi { AM_OF_SHIFT }
        else { MM_OF_SHIFT };
        self.order_mem[pos] |= 1 << shift;
    }

    fn off_of_flag(&mut self, pos: usize, om_type: OrdMemType) {
        let shift = 
        if om_type == OrdMemType::Addi { AM_OF_SHIFT }
        else { MM_OF_SHIFT };
        self.order_mem[pos] &= !(1 << shift);
    }

    #[inline(always)]
    fn get_of_flag(&self, pos: usize, om_type: OrdMemType) -> u8 {
        let shift = 
        if om_type == OrdMemType::Addi { AM_OF_SHIFT }
        else { MM_OF_SHIFT };

        return (self.order_mem[pos] >> shift) & 1
    }

    #[inline(always)]
    fn set_amount(&mut self, pos: usize, amount: u8, om_type: OrdMemType) {
        let (shift, mask) = 
        if om_type == OrdMemType::Addi { (AM_AMOUNT_SHIFT, AM_AMOUNT_MASK) }
        else { (MM_AMOUNT_SHIFT, MM_AMOUNT_MASK) };

        self.order_mem[pos] &= !mask;
        self.order_mem[pos] |= amount << shift;
    }

    #[inline(always)]
    fn get_amount(&self, pos: usize, om_type: OrdMemType) -> u8 {
        let shift = 
        if om_type == OrdMemType::Addi { AM_AMOUNT_SHIFT }
        else { MM_AMOUNT_SHIFT };

        return (self.order_mem[pos] >> shift) & AMOUNT_MAX
    }
}
// [-] inline fn
// --------------------------------------------


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
        self.trig_cur_am = false;
        self.om_mm_pos += 1;
        if self.om_mm_pos > self.om_am_end_pos { self.order_mem.push(0x00); } // init cell, in real dev it already 0x00 
        if self.om_mm_pos > self.om_mm_end_pos { self.om_mm_end_pos = self.om_mm_pos; }
    }

    fn om_inc_am_pos(&mut self) {
        self.om_am_end_pos += 1;
        if self.om_am_end_pos > self.om_mm_end_pos { self.order_mem.push(0x00); } // init cell, in real dev it already 0x00 
    }

    fn om_inc_mm_con(&mut self) {
        let pos = self.om_mm_pos;
        let om_type = OrdMemType::Main;

        let mut x = self.get_amount(pos, om_type);

        if x == AMOUNT_MAX { 
            self.on_of_flag(pos, om_type);
            self.om_inc_mm_pos();
            x = 0;
        }

        let x = x + 1;
        self.set_amount(self.om_mm_pos, x, om_type);
        self.reg_con_amount = x;
    }
    
    fn om_inc_am_con(&mut self) {
        let pos = self.om_am_end_pos;
        let om_type = OrdMemType::Addi;

        let mut x = self.get_amount(pos, om_type);
        if x == AMOUNT_MAX {
            self.on_of_flag(pos, om_type);
            self.om_inc_am_pos();
            x = 0;
        } 
        let x = x + 1;
        self.set_amount(self.om_am_end_pos, x, om_type);
    }

    fn om_mm_transfer(&mut self) {
        let pos = self.om_mm_pos;
        let om_type = OrdMemType::Main;
        
        let amount = self.get_amount(pos, om_type);
        let transfer = amount - self.reg_con_amount; 

        let end_is_other = self.get_of_flag(pos, om_type) == 1;
        if end_is_other && self.om_mm_pos == self.om_mm_end_pos { panic!("[ALGO ERROR] it cant be in the same time in two dif state") }
        if !end_is_other && self.om_mm_pos != self.om_mm_end_pos { panic!("[ALGO ERROR] it cant be in the same time in two dif state") }

        if transfer == 0 { 
            // if (T7XX & reg_con_amount == 7) => F7XX
            self.off_of_flag(pos, om_type);
            return
        }

        self.off_of_flag(pos, om_type);
        self.set_amount(pos, self.reg_con_amount, om_type);

        if end_is_other {
            let pos = self.om_mm_end_pos;
            let end_amount = self.get_amount(pos, om_type);
            
            let new_val = transfer + end_amount;
            if new_val > AMOUNT_MAX {
                self.on_of_flag(pos, om_type);
                self.set_amount(pos, AMOUNT_MAX, om_type);
                let new_val = new_val - AMOUNT_MAX;

                if new_val >= AMOUNT_MAX { panic!("[ALGO ERROR] : transfer + end_amount must be < 2*AMOUNT_MAX") }
                
                self.om_mm_end_pos += 1;
                if self.om_mm_end_pos > self.om_am_end_pos { 
                    // init cell, in real dev it already 0x00
                    self.order_mem.push(Self::create_om_cell(false, new_val, false, 0)); 
                } else {
                    let pos = self.om_mm_end_pos;
                    self.set_amount(pos, new_val, om_type);
                }
            } else {
                self.set_amount(pos, new_val, om_type);
            }
        } else {
            let pos = self.om_mm_end_pos;
            if self.get_of_flag(pos, om_type) == 1 { algo_error!() }

            self.om_mm_end_pos += 1;
            if self.om_mm_end_pos > self.om_am_end_pos { self.order_mem.push(0x00); } // init cell, in real dev it already 0x00 
            let pos = self.om_mm_end_pos;
            self.set_amount(pos, transfer, om_type);
        }
    }

    fn om_mm_glue(&mut self) {
        if self.om_mm_pos == self.om_mm_end_pos { return }

        let two_in_a_row = self.om_mm_pos + 1 == self.om_mm_end_pos;

        let cur_byte = self.order_mem[self.om_mm_pos];
        let end_byte = self.order_mem[self.om_mm_end_pos];
        
        if (cur_byte >> MM_OF_SHIFT) & 1 == 1 { algo_error!() }

        let max_move = AMOUNT_MAX - ((cur_byte >> MM_AMOUNT_SHIFT) & AMOUNT_MAX);
        let end_can_move = (end_byte >> MM_AMOUNT_SHIFT) & AMOUNT_MAX;

        if end_can_move > max_move {
            self.order_mem[self.om_mm_pos] = cur_byte | (1 << MM_OF_SHIFT) | (AMOUNT_MAX << MM_AMOUNT_SHIFT); 
            self.order_mem[self.om_mm_end_pos] = (end_byte & !MM_AMOUNT_MASK) | ((end_can_move - max_move) << MM_AMOUNT_SHIFT);
            return 
        }
        if end_can_move == max_move {
            self.order_mem[self.om_mm_pos] = cur_byte | (1 << MM_OF_SHIFT) | (AMOUNT_MAX << MM_AMOUNT_SHIFT);  
            if two_in_a_row { self.order_mem[self.om_mm_pos] &= !(1 << MM_OF_SHIFT); }
            self.order_mem[self.om_mm_end_pos] &= !MM_AMOUNT_MASK;
            if self.order_mem[self.om_mm_end_pos] == 0x00 { self.order_mem.pop(); }
            self.om_mm_end_pos -= 1;
            self.order_mem[self.om_mm_end_pos] &= !(1 << MM_OF_SHIFT);
            return
        }

        // end_can_move < max_move
        if two_in_a_row {
            let delta = max_move - end_can_move;
            self.order_mem[self.om_mm_pos] = (cur_byte & !MM_AMOUNT_MASK) | ((AMOUNT_MAX - delta) << MM_AMOUNT_SHIFT); 
            self.order_mem[self.om_mm_end_pos] &= !MM_AMOUNT_MASK;
            if self.order_mem[self.om_mm_end_pos] == 0x00 { self.order_mem.pop(); }
            self.om_mm_end_pos -= 1;
        } else {
            let delta = max_move - end_can_move;
            self.order_mem[self.om_mm_pos] = (cur_byte & !MM_AMOUNT_MASK) | (AMOUNT_MAX << MM_AMOUNT_SHIFT) | (1 << MM_OF_SHIFT); 
            self.order_mem[self.om_mm_end_pos] &= !MM_AMOUNT_MASK;
            if self.order_mem[self.om_mm_end_pos] == 0x00 { self.order_mem.pop(); }
            self.om_mm_end_pos -= 1;
            if (self.order_mem[self.om_mm_end_pos] >> MM_AMOUNT_SHIFT) != 0x0F { panic!("[ALGO ERROR]") } 
            self.order_mem[self.om_mm_end_pos] &= !(1 << MM_OF_SHIFT);
            self.order_mem[self.om_mm_end_pos] &= !MM_AMOUNT_MASK;
            self.order_mem[self.om_mm_end_pos] |=  (AMOUNT_MAX - delta) << MM_AMOUNT_SHIFT;
        }
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
            am_last_clear_pos: None,
            trig_am_can_create: true,

            order_mem: om,
            om_size,

            om_mm_end_pos: 0,
            om_mm_pos: 0,
            om_am_end_pos: 0,
            om_am_pos: 0,
            trig_cur_am: false, // first mem-cell always is mm even in run-time

            reg_con_amount: 1,

            error: (mm_size == 0),
        }
    }

    fn cur_con_end(&self) -> bool {
        let con_amount_real = 
            if self.trig_cur_am { self.get_amount(self.om_am_pos, OrdMemType::Addi) } 
            else { self.get_amount(self.om_mm_pos, OrdMemType::Main) };
        if con_amount_real < self.reg_con_amount { panic!("[ALGO ERROR] amount_real = {} < reg_amount = {}", con_amount_real, self.reg_con_amount) } 
        con_amount_real == self.reg_con_amount
    }

    fn stay_on_the_end(&self) -> bool {
        let am_can_be_end = 
            if self.am_pos.is_none() { 
                if self.om_am_pos != 0 { panic!("[ALGO ERROR]") };
                (self.am_size == 0) || (self.get_amount(0, OrdMemType::Addi) == 0)
            } else {
                (self.am_pos.is_some() && self.am_pos.unwrap() == (self.am_size - 1))
                ||
                (self.order_mem.len() == self.om_am_pos + 1)
                ||
                (self.get_amount(self.om_am_pos + 1, OrdMemType::Addi) == 0)
            };
        
        let mm_can_be_end = (self.mm_pos == (self.mm_size - 1))
            || (self.order_mem.len() == self.om_mm_pos + 1)
            || (self.get_amount(self.om_mm_pos + 1, OrdMemType::Main) == 0);

        let con_end = self.cur_con_end();

        am_can_be_end && mm_can_be_end && con_end
    }

    fn stay_on_the_start(&self) -> bool {
        if self.trig_cur_am { false } 
        else { self.reg_con_amount == 1 && self.mm_pos == 0 }
    }

    fn cur_con_overflow(&self) -> bool {
        if self.trig_cur_am { self.get_of_flag(self.om_am_pos, OrdMemType::Addi) == 1 } 
        else { self.get_of_flag(self.om_mm_pos, OrdMemType::Main) == 1 }
    }

    fn cur_con_prev_overflow(&self) -> bool {
        if self.trig_cur_am { 
            let om_am_pos = self.om_am_pos;
            (om_am_pos > 0) && (self.get_of_flag(om_am_pos - 1, OrdMemType::Addi) == 1)
        } else {
            let om_mm_pos = self.om_mm_pos;
            (om_mm_pos > 0) && (self.get_of_flag(om_mm_pos - 1, OrdMemType::Main) == 1)
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

        if on_con_end && !self.trig_cur_am && (self.om_mm_pos == self.om_mm_end_pos) && (self.get_value().unwrap() == 0x00) {
            // we just dec counter, so it take ~0 overhead & easier than 
            //                      save amount of already pushed for not push more than need: 
            self.main_mem.pop(); 

            if self.reg_con_amount == 0 {
                self.off_of_flag(self.om_mm_pos, OrdMemType::Main);
                self.set_amount(self.om_mm_pos, 0, OrdMemType::Main);
                if self.order_mem[self.om_mm_pos] == 0x00 { self.order_mem.pop(); }
            } else {
                self.set_amount(self.om_mm_pos, self.reg_con_amount, OrdMemType::Main);
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
            
            if self.trig_cur_am { self.om_inc_mm_pos(); }
            self.om_inc_mm_con(); 
        } else {
            if self.cur_con_end() {
                self.reg_con_amount = 1;
                let overflow = self.cur_con_overflow();
                if !overflow { self.trig_cur_am = !self.trig_cur_am; }
                if self.trig_cur_am {
                    self.am_pos = 
                        if let Some(x) = self.am_pos {
                            self.om_am_pos += 1;
                            Some(x + 1) 
                        } 
                        else { 
                            if overflow { panic!("[ALGO ERROR]: cause cur is overflow | mm is not end => next already exist") };
                            Some(0) 
                        };
                    if self.am_last_clear_pos.is_none() { self.am_last_clear_pos = self.am_pos; }
                } else {
                    self.mm_pos += 1;
                    self.om_mm_pos += 1;
                }
            } else {
                self.reg_con_amount += 1;
                if self.trig_cur_am { 
                    self.am_pos = 
                        if let Some(x) = self.am_pos { Some(x + 1) } 
                        else { panic!("[ALGO ERROR]: cause cur is not con_end => next already exist") }; 
                }
                else { self.mm_pos += 1; }
            }
            if self.trig_cur_am && self.am_pos == self.am_last_clear_pos { self.trig_am_can_create = true }
            if self.om_am_end_pos < self.om_am_pos { self.om_am_end_pos = self.om_am_pos; }
        }
    }

    pub fn prev_cell(&mut self) {
        if self.error { return }

        let on_con_end = self.cur_con_end();
        let on_the_end = self.stay_on_the_end();

        if self.trig_cur_am && self.am_last_clear_pos.is_some() && self.am_pos == self.am_last_clear_pos { self.trig_am_can_create = false }

        if self.reg_con_amount == 1 {
            if self.stay_on_the_start() { self.error = true; return }

            self.reg_con_amount = 0;
            // [NOT NECESSARY]
            let need_null_mm_of = on_the_end && self.try_pseudo_mm_del(on_con_end);

            let overflow = self.cur_con_prev_overflow();

            if self.trig_cur_am {
                if self.am_pos == Some(0) { self.am_pos = None; } 
                else { 
                    self.am_pos = Some(self.am_pos.unwrap() - 1); 
                    self.om_am_pos -= 1;
                }
            } else {
                self.mm_pos -= 1;
                self.om_mm_pos -= 1;
                // if need nullify MM OF-flag:
                if need_null_mm_of { 
                    self.off_of_flag(self.om_mm_pos, OrdMemType::Main);
                    self.om_mm_end_pos -= 1;
                }
            }      

            if !overflow {  self.trig_cur_am = !self.trig_cur_am; }
            
            let (pos, sh) = if self.trig_cur_am { (self.om_am_pos, AM_AMOUNT_SHIFT) } else { (self.om_mm_pos, MM_AMOUNT_SHIFT) };
            self.reg_con_amount = (self.order_mem[pos] >> sh) & AMOUNT_MAX;
        } else {
            self.reg_con_amount -= 1;
            if self.trig_cur_am { 
                self.am_pos = 
                    if let Some(x) = self.am_pos { Some(x - 1) } 
                    else { panic!("[ALGO ERROR]: cause cur is not con_start => prev exist") }; 
            } else {
                // [NOT NECESSARY]
                if on_the_end { self.try_pseudo_mm_del(on_con_end); }

                self.mm_pos -= 1;
            }
        }
    }

    pub fn set_value(&mut self, value: u8) {
        if self.error { return }
        if self.trig_cur_am { 
            self.additional_mem[self.am_pos.unwrap()] = value; 
            if self.trig_am_can_create {
                if !(self.am_pos == self.am_last_clear_pos) {
                    self.am_last_clear_pos = self.am_pos;
                    //self.trig_am_can_create = false;
                }
            }
        } else { 
            self.main_mem[self.mm_pos] = value; 
        }
    }

    pub fn get_value(&self) -> Option<u8> {
        if self.error { return None }
        if self.trig_cur_am { Some(self.additional_mem[self.am_pos.unwrap()]) }
        else { Some(self.main_mem[self.mm_pos]) }
    }

    pub fn create_cell(&mut self) {
        if self.error { return }
        if !self.trig_am_can_create { self.error = true; return }
        if (self.am_pos.is_none() && self.am_size == 0) 
            || (self.am_pos.is_some() && self.am_pos.unwrap() == (self.am_size - 1))
            { self.error = true; return }
        
        self.additional_mem.push(0x00);

        let start_on_mm = !self.trig_cur_am;
        
        let con_amount_save = self.reg_con_amount;
        if start_on_mm { 
            self.om_mm_transfer();
            if self.am_pos.is_some() {
                self.om_am_pos += 1;
                self.om_am_end_pos += 1;
                if self.om_am_pos != self.om_am_end_pos { algo_error!() }
                if self.om_am_pos > self.om_mm_end_pos { self.order_mem.push(0x00); }
                self.set_amount(self.om_am_pos, 1, OrdMemType::Addi);
                self.om_am_pos -= 1;
            } else {
                if self.om_am_pos != 0 { algo_error!() }
                self.set_amount(0, 1, OrdMemType::Addi);
            }
        } else {
            self.om_inc_am_con();
        }
        self.reg_con_amount = con_amount_save;

        self.next_cell();
        if start_on_mm { self.am_last_clear_pos = self.am_pos; }
    }

    pub fn delete_cell(&mut self) {
        if self.error { return }
        if !self.trig_cur_am { self.error = true; return }
        if !self.trig_am_can_create { self.error = true; return }

        let om_type = OrdMemType::Addi;

        let am_pos = self.am_pos.unwrap();
        let last_clear_am_pos = self.am_last_clear_pos.unwrap();

        if last_clear_am_pos > am_pos { panic!("[ALGO ERROR]"); } //trig_cur_am
        
        if last_clear_am_pos < am_pos {
            // we have clear created-cell after cur pos, so just pop it:
            if self.additional_mem.pop() != Some(0x00) { panic!("[ALGO ERROR]") }
            
            self.prev_cell();
            if !self.trig_cur_am { panic!("[ALGO ERROR]") }
            
            let pos = self.om_am_end_pos;
            if self.get_of_flag(pos, om_type) == 1 { panic!("[ALGO ERROR]") }
            let x = self.get_amount(pos, om_type);
            if x == 0 { panic!("[ALGO ERROR]") }

            if x == 1 {
                self.set_amount(pos, 0, om_type);
                if self.order_mem[pos] == 0x00 { self.order_mem.pop(); }
                if pos == 0 { panic!("[ALGO ERROR]") }

                let pos = pos - 1;
                if self.get_amount(pos, om_type) != AMOUNT_MAX { panic!("[ALGO ERROR]") }
                self.off_of_flag(pos, om_type);

                if self.om_am_pos == self.om_am_end_pos { self.om_am_pos -= 1; }
                self.om_am_end_pos -= 1;
            } else {
                self.set_amount(pos, x - 1, om_type);
            }

            self.am_pos = Some(am_pos - 1);
        } else {
            /*
            self.next_cell();
            let stay_end = !self.trig_cur_am;
            self.prev_cell();
            */

            self.additional_mem[am_pos] = 0x00;
            self.prev_cell();
            let stay_first = !self.trig_cur_am;

            let mut need_mm_glue = false;

            let pos = self.om_am_end_pos;
            let x = self.get_amount(pos, om_type);
            if x == 1 {
                self.set_amount(pos, 0, om_type);
                if self.order_mem[pos] == 0x00 { self.order_mem.pop(); }
                if pos != 0 {
                    let pos = pos - 1;
                    if self.get_of_flag(pos, om_type) == 0 { need_mm_glue = true; } 
                    self.off_of_flag(pos, om_type);

                    self.om_am_end_pos -= 1;
                    if self.om_am_pos > self.om_am_end_pos { self.om_am_pos -= 1; }
                } else {
                    need_mm_glue = true;
                }
            } else {
                self.set_amount(pos, x - 1, om_type);
            }

            if need_mm_glue { self.om_mm_glue(); } 

            self.additional_mem.pop();
            if stay_first && !need_mm_glue { self.next_cell(); }

            //self.am_pos = if am_pos == 0 { None } else { Some(am_pos - 1) };
            self.am_last_clear_pos = self.am_pos;
            self.trig_am_can_create = true;
        }
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
            if (new_line_each > 0) && ind != 0 && (ind % new_line_each == 0) { ret += "\n|"; } 
        }
        ret += "F0F0|...";
        ret
    } 

    pub fn print_mm(&self, new_line_each: usize) -> String {
        let mut ret = String::new();
        ret.push('|');
        for (ind, x) in (&self.main_mem).into_iter().enumerate() {
            ret += &format!("{:02X}|", x);
            if (new_line_each > 0) && ind != 0 && (ind % new_line_each == 0) { ret += "\n|"; } 
        }
        ret += "00|...";
        ret
    }
    
    pub fn print_am(&self, new_line_each: usize) -> String {
        let mut ret = String::new();
        ret.push('|');
        for (ind, x) in (&self.additional_mem).into_iter().enumerate() {
            ret += &format!("{:02X}|", x);
            if (new_line_each > 0) && ind != 0 && (ind % new_line_each == 0) { ret += "\n|"; } 
        }
        ret += "00|...";
        ret
    }
}
// [-] PRINT
// ---------------------------------------------------------



// |-----------------------------------------------|-------|
// |        TTTTT   EEEEE      SSSS    TTTTT       |   ∥   |
// |          T     E         S          T         |   ∥   |
// |          T     EEEEE      SSS       T         |   ∥   |
// |          T     E             S      T         |  \∥/  |
// |          T     EEEEE     SSSS       T         |   V   |
// |-----------------------------------------------|-------|


#[cfg(test)]
pub mod tests {
    use super::*;

    pub(in super) fn full_print(cem: &CemInner) {
        println!("+++ ↓↓↓ ↓↓↓ ↓↓↓ +++");  
        println!("{} ERROR", if cem.error() { "!!!" } else { "NO" });
        println!("reg con amount = {}", cem.reg_con_amount);
        println!("STAY ON [{}]", if cem.trig_cur_am { "AM" } else { "MM" });
        println!("CAN CREATE = {}", cem.trig_am_can_create);
        println!("PTR[AM]: cur_ptr = {:?}; last_clear_ptr = {:?};", cem.am_pos, cem.am_last_clear_pos);
        println!("PTR[OM]: am = {}; am_end = {}; mm = {}; mm_end = {};", cem.om_am_pos, cem.om_am_end_pos, cem.om_mm_pos, cem.om_mm_end_pos);
        println!("OM: {}", cem.print_om(0));
        println!("MM: {}", cem.print_mm(0));
        println!("AM: {}", cem.print_am(0));  
        println!("--- ↑↑↑ ↑↑↑ ↑↑↑ ---");  
    }

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
        assert_eq!(&cem.print_om(0), "|F7F0|F0F0|...");
        assert_eq!(&cem.print_mm(0), "|00|02|00|03|01|00|04|00|...");
        assert_eq!(cem.error(), false);

        println!("OM: {}", cem.print_om(0));
        println!("MM: {}", cem.print_mm(0));
        println!("AM: {}", cem.print_am(0));
    }

    #[test]
    fn test_cem_02(){
        let const_mem = "|00|01|02|03|04|05|06|07|08|09|0A|0B|0C|0D|0E|0F|10|11|12|13|14|15|16|17|18|19|1A|1B|1C|1D|1E|1F|00|...";
        let mut cem = CemInner::new(1024, 1024);
        for i in 0..31 {
            cem.next_cell();
            cem.set_value(i + 1);
        }
        assert_eq!(&cem.print_om(0), "|T7F0|T7F0|T7F0|T7F0|F4F0|F0F0|...");
        for _ in 0..16 { cem.prev_cell(); }
        
        cem.create_cell();
        cem.set_value(0xAC);
        assert_eq!(&cem.print_om(0), "|T7F1|T7F0|F2F0|T7F0|T7F0|F2F0|F0F0|...");
        assert_eq!(&cem.print_am(0), "|AC|00|...");
        cem.create_cell();
        cem.set_value(0xAB);
        assert_eq!(&cem.print_om(0), "|T7F2|T7F0|F2F0|T7F0|T7F0|F2F0|F0F0|...");
        assert_eq!(&cem.print_am(0), "|AC|AB|00|...");
        assert_eq!(&cem.print_mm(0), const_mem);
        
        cem.create_cell();
        cem.prev_cell();
        cem.create_cell();
        cem.prev_cell();
        cem.create_cell();
        assert_eq!(&cem.print_am(0), "|AC|AB|00|00|00|00|...");
        assert_eq!(cem.am_last_clear_pos, Some(1));
        assert_eq!(cem.am_pos, Some(2));
        assert_eq!(cem.trig_am_can_create, true);
        cem.prev_cell();
        cem.prev_cell();
        assert_eq!(cem.trig_am_can_create, false);
        cem.next_cell();
        assert_eq!(cem.am_pos, Some(1));
        assert_eq!(cem.trig_am_can_create, true);
        cem.prev_cell();
        assert_eq!(cem.am_pos, Some(0));
        assert_eq!(cem.trig_cur_am, true);
        cem.prev_cell();
        assert_eq!(cem.am_pos, None);
        assert_eq!(cem.trig_cur_am, false);
        cem.prev_cell();
        assert_eq!(cem.am_pos, None);
        assert_eq!(cem.trig_cur_am, false);
        for _ in 0..4 { 
            cem.prev_cell(); 
            assert_eq!(cem.trig_cur_am, false);
        }
        for _ in 0..4 { 
            cem.next_cell(); 
            assert_eq!(cem.trig_cur_am, false);
        }
        cem.next_cell();
        cem.next_cell();
        assert_eq!(cem.trig_cur_am, true);
        assert_eq!(cem.am_pos, Some(0));
        assert_eq!(cem.trig_am_can_create, false);
        cem.next_cell(); 
        cem.next_cell(); 
        assert_eq!(cem.trig_am_can_create, true);
        for _ in 0..2 { cem.next_cell(); }
        assert_eq!(cem.trig_cur_am, true);
        cem.create_cell();
        assert_eq!(cem.am_last_clear_pos, Some(1));
        cem.set_value(0xFF);
        assert_eq!(cem.am_last_clear_pos, Some(5));
        assert_eq!(&cem.print_am(0), "|AC|AB|00|00|00|FF|00|...");
        assert_eq!(&cem.print_om(0), "|T7F6|T7F0|F2F0|T7F0|T7F0|F2F0|F0F0|...");
        
        cem.next_cell();
        assert_eq!(cem.am_pos, Some(5));
        assert_eq!(cem.trig_cur_am, false);
        assert_eq!(cem.get_value(), Some(0x10));
        cem.set_value(0x10);
        assert_eq!(&cem.print_mm(0), const_mem);
        for _ in 0..0x0F { 
            cem.next_cell();
            assert_eq!(cem.trig_cur_am, false);
            assert_eq!(cem.trig_am_can_create, true);
        }
        assert_eq!(cem.am_pos, Some(5));
        assert_eq!(cem.get_value(), Some(0x1F));
        assert_eq!(cem.stay_on_the_end(), true);

        for _ in 0..0x0F { cem.prev_cell(); }
        assert_eq!(cem.trig_cur_am, false);
        assert_eq!(cem.trig_am_can_create, true);
        assert_eq!(cem.get_value(), Some(0x10));
        cem.prev_cell();
        assert_eq!(cem.trig_cur_am, true);
        assert_eq!(cem.trig_am_can_create, true);
        assert_eq!(cem.get_value(), Some(0xFF));
        for _ in 0..5 { 
            cem.prev_cell(); 
            assert_eq!(cem.trig_am_can_create, false);
        }
        assert_eq!(cem.trig_cur_am, true);
        assert_eq!(cem.get_value(), Some(0xAC));
        cem.prev_cell();
        assert_eq!(cem.trig_cur_am, false);
        assert_eq!(cem.get_value(), Some(0x0F));

        for _ in 0..0x0F { cem.prev_cell(); }
        assert_eq!(cem.stay_on_the_start(), true);

        for _ in 0..(0x0F + 1 + 5 + 1 + 5) { cem.next_cell(); }
        assert_eq!(cem.get_value(), Some(0x15));
        assert_eq!(cem.trig_cur_am, false);
        assert_eq!(cem.trig_am_can_create, true);

        for i in 0..10 {
            cem.create_cell();
            cem.set_value(10 - i);
        }
        assert_eq!(&cem.print_om(0), "|T7F6|T7T7|F2F3|F6F0|T7F0|F3F0|F0F0|...");
        assert_eq!(&cem.print_am(0),  "|AC|AB|00|00|00|FF|0A|09|08|07|06|05|04|03|02|01|00|...");
        assert_eq!(&cem.print_mm(0), const_mem);
        assert_eq!(cem.stay_on_the_start(), false);
        assert_eq!(cem.stay_on_the_end(), false);

        // go to mem end:
        for _ in 0x16..0x20 { cem.next_cell(); } 
        assert_eq!(cem.stay_on_the_end(), true);

        // check all memory when go backward:
        for x in (0x16..0x20).rev() { 
            assert_eq!(cem.get_value(), Some(x));
            cem.prev_cell(); 
        }
        for x in 0..10 { 
            assert_eq!(cem.get_value(), Some(x + 1));
            cem.prev_cell(); 
        }
        for x in (0x10..=0x15).rev() { 
            assert_eq!(cem.get_value(), Some(x));
            cem.prev_cell(); 
        }
        assert_eq!(cem.get_value(), Some(0xFF));
        cem.prev_cell(); 
        for _ in 0..3 { 
            assert_eq!(cem.get_value(), Some(0x00));
            cem.prev_cell(); 
        }
        let ab = cem.get_value().unwrap();
        cem.prev_cell();
        let ac = cem.get_value().unwrap();
        cem.prev_cell(); 
        assert_eq!((ac, ab), (0xAC, 0xAB));
        for x in (0x01..0x10).rev() { 
            assert_eq!(cem.get_value(), Some(x));
            cem.prev_cell(); 
        }
        assert_eq!(cem.get_value(), Some(0x00));
        assert_eq!(cem.stay_on_the_start(), true);

        // check all memory when go forward:
        for x in 0x00..0x10 { 
            assert_eq!(cem.get_value(), Some(x));
            cem.next_cell(); 
        }
        let ac = cem.get_value().unwrap();
        cem.next_cell();
        let ab = cem.get_value().unwrap();
        cem.next_cell(); 
        assert_eq!((ac, ab), (0xAC, 0xAB));
        for _ in 0..3 { 
            assert_eq!(cem.get_value(), Some(0x00));
            cem.next_cell(); 
        }
        assert_eq!(cem.get_value(), Some(0xFF));
        cem.next_cell();
        for x in 0x10..=0x15 { 
            assert_eq!(cem.get_value(), Some(x));
            cem.next_cell(); 
        }
        for x in 0..10 { 
            assert_eq!(cem.get_value(), Some(10 - x));
            cem.next_cell(); 
        }
        for x in 0x16..0x20 { 
            assert_eq!(cem.get_value(), Some(x));
            cem.next_cell(); 
        }
        assert_eq!(cem.get_value(), Some(0x00));
        cem.prev_cell(); 


        assert_eq!(cem.error(), false);
        assert_eq!(cem.print_mm(0), const_mem);
        full_print(&cem);
    }

    #[test]
    fn test_cem_03(){
        let mut cem = CemInner::new(1024, 1024);
        for _ in 0..3 { 
            cem.create_cell();
            cem.next_cell();
        }

        assert_eq!(&cem.print_om(0), "|F1F1|F1F1|F1F1|F1F0|F0F0|...");

        for _ in 0..3 { 
            cem.prev_cell(); 
            cem.prev_cell(); 
        }
        assert_eq!(&cem.print_om(0), "|F1F1|F1F1|F1F1|F0F0|...");
        assert_eq!(&cem.print_mm(0), "|00|00|00|00|...");
        assert_eq!(&cem.print_am(0), "|00|00|00|00|...");

        for _ in 0..6 { cem.next_cell(); }
        assert_eq!(&cem.print_om(0), "|F1F1|F1F1|F1F1|F1F0|F0F0|...");
        assert_eq!(&cem.print_mm(0), "|00|00|00|00|00|...");
        assert_eq!(&cem.print_am(0), "|00|00|00|00|...");

        for _ in 0..4 { cem.next_cell(); }
        assert_eq!(&cem.print_om(0), "|F1F1|F1F1|F1F1|F5F0|F0F0|...");
        assert_eq!(&cem.print_mm(0), "|00|00|00|00|00|00|00|00|00|...");
        assert_eq!(&cem.print_am(0), "|00|00|00|00|...");

        for _ in 0..4 { cem.prev_cell(); }
        assert_eq!(&cem.print_om(0), "|F1F1|F1F1|F1F1|F1F0|F0F0|...");
        assert_eq!(&cem.print_mm(0), "|00|00|00|00|00|...");
        assert_eq!(&cem.print_am(0), "|00|00|00|00|...");

        for _ in 0..4 { cem.prev_cell(); }
        assert_eq!(&cem.print_om(0), "|F1F1|F1F1|F1F1|F0F0|...");
        assert_eq!(&cem.print_mm(0), "|00|00|00|00|...");
        assert_eq!(&cem.print_am(0), "|00|00|00|00|...");

        assert_eq!(cem.error(), false);
        full_print(&cem);
    }

    #[test]
    fn test_cem_04(){
        let mut cem = CemInner::new(1024, 1024);
        cem.set_value(0xFF);
        cem.next_cell();
        cem.set_value(0xFF);
        cem.next_cell();
        cem.set_value(0xFF);
        assert_eq!(cem.stay_on_the_end(), true);

        cem.prev_cell();
        cem.prev_cell();

        for x in 0..15 { 
            cem.create_cell();
            cem.set_value(0xA0 + x);
            assert_eq!(cem.stay_on_the_end(), false);
        }
        cem.next_cell();
        for x in 0..15 { 
            cem.create_cell();
            cem.set_value(0xB0 + x);
            assert_eq!(cem.stay_on_the_end(), false);
        }
        cem.next_cell();
        assert_eq!(cem.stay_on_the_end(), true);
        
        for _ in 0..(15+15+2) { cem.prev_cell(); } 
        assert_eq!(cem.stay_on_the_start(), true);
        
        assert_eq!(cem.get_value(), Some(0xFF));
        for x in 0..15 {
            cem.next_cell();
            assert_eq!(cem.get_value(), Some(0xA0 + x));
        }
        cem.next_cell();
        assert_eq!(cem.get_value(), Some(0xFF));
        for x in 0..15 {
            cem.next_cell();
            assert_eq!(cem.get_value(), Some(0xB0 + x));
        }
        cem.next_cell();
        assert_eq!(cem.get_value(), Some(0xFF));
        assert_eq!(cem.stay_on_the_end(), true);


        assert_eq!(cem.error(), false);
        assert_eq!(&cem.print_om(0), "|F1T7|F1T7|F1F1|F0T7|F0T7|F0F1|F0F0|...");
        assert_eq!(&cem.print_mm(0), "|FF|FF|FF|00|...");
        assert_eq!(&cem.print_am(0), "|A0|A1|A2|A3|A4|A5|A6|A7|A8|A9|AA|AB|AC|AD|AE|B0|B1|B2|B3|B4|B5|B6|B7|B8|B9|BA|BB|BC|BD|BE|00|...");
        full_print(&cem);
    }

    #[test]
    fn test_cem_05(){
        let mut cem = CemInner::new(1024, 1024);
        cem.create_cell();
        for ind in 2..25 {
            cem.create_cell();
            cem.prev_cell();
            assert_eq!(cem.stay_on_the_end(), false);
            assert_eq!(cem.om_am_pos, 0);
            assert_eq!(cem.om_am_end_pos, ((ind - 1) / AMOUNT_MAX) as usize);
            assert_eq!(cem.om_mm_pos, 0);
            assert_eq!(cem.om_mm_end_pos, 0);
            if ind <= AMOUNT_MAX { assert_eq!(cem.print_om(0), format!("|F1F{}|F0F0|...", ind)); } 
            else if ind <= AMOUNT_MAX * 2 { assert_eq!(cem.print_om(0), format!("|F1T7|F0F{}|F0F0|...", ind - AMOUNT_MAX)); } 
            else if ind <= AMOUNT_MAX * 3 { assert_eq!(cem.print_om(0), format!("|F1T7|F0T7|F0F{}|F0F0|...", ind - AMOUNT_MAX * 2)); } 
            else if ind <= AMOUNT_MAX * 4 { assert_eq!(cem.print_om(0), format!("|F1T7|F0T7|F0T7|F0F{}|F0F0|...", ind - AMOUNT_MAX * 3)); } 
        }
        full_print(&cem);
    }

    fn test_cem_06_helper(version: usize) {
        let mut cem = CemInner::new(1024, 1024);
        cem.set_value(0x01);
        cem.next_cell();
        cem.set_value(0x02);
        cem.next_cell();
        cem.set_value(0x04);
        cem.next_cell();
        cem.set_value(0x08);
        loop { 
            cem.prev_cell();
            if let Some(0x01) = cem.get_value() { break; }
        }

        cem.create_cell();
        cem.set_value(0xAC);
        cem.create_cell();
        cem.set_value(0xAB);

        cem.next_cell();
        cem.next_cell();

        cem.create_cell();
        cem.set_value(0x1C);

        cem.delete_cell();
        assert_eq!(cem.get_value(), Some(0x04));
        cem.next_cell();
        assert_eq!(cem.get_value(), Some(0x08));
        cem.prev_cell();

        cem.create_cell();
        cem.create_cell();
        cem.prev_cell();
        cem.create_cell();
        cem.set_value(0x1C);
        cem.prev_cell();
        cem.set_value(0xC1);
        assert_eq!(&cem.print_om(0), "|F1F2|F2F3|F1F0|F0F0|...");
        assert_eq!(cem.error(), false);
        // LOL we really cant! it's first catched by cem error! : cem.delete_cell();
        if version == 1 {
            cem.delete_cell();
            assert_eq!(cem.error(), true);
            return
        }
        cem.next_cell();
        assert_eq!(cem.get_value(), Some(0x1C));
        cem.delete_cell();
        assert_eq!(cem.get_value(), Some(0xC1));
        cem.create_cell();

        if version == 2 || version == 3 || version == 4 || version == 5 {
            cem.prev_cell();
            cem.prev_cell();
            cem.prev_cell();
            cem.prev_cell();
            assert_eq!(cem.error(), false);
            assert_eq!(cem.get_value(), Some(0xAB));
            if version == 2 { cem.create_cell(); }
            if version == 3 { cem.delete_cell(); }
            if version == 4  || version == 5 { 
                cem.next_cell();
                assert_eq!(cem.get_value(), Some(0x02));
                if version == 4 { cem.delete_cell(); }
                else { cem.create_cell(); } 
            }
            assert_eq!(cem.error(), true);
            return
        }

        cem.next_cell();
        cem.create_cell();
        cem.delete_cell();

        cem.delete_cell();
        assert_eq!(cem.get_value(), Some(0x00));
        cem.delete_cell();
        assert_eq!(cem.get_value(), Some(0xC1));
        assert_eq!(&cem.print_om(0), "|F1F2|F2F1|F1F0|F0F0|...");
        cem.delete_cell();
        assert_eq!(&cem.print_om(0), "|F1F2|F3F0|F0F0|...");
        assert_eq!(cem.get_value(), Some(0x04));
        cem.next_cell();
        assert_eq!(cem.get_value(), Some(0x08));
        cem.prev_cell();

        cem.prev_cell();
        cem.prev_cell();

        assert_eq!(cem.get_value(), Some(0xAB));
        cem.delete_cell();
        assert_eq!(cem.get_value(), Some(0xAC));
        assert_eq!(&cem.print_om(0), "|F1F1|F3F0|F0F0|...");
        cem.delete_cell();
        assert_eq!(&cem.print_om(0), "|F4F0|F0F0|...");
        assert_eq!(cem.get_value(), Some(0x01));

        assert_eq!(cem.error(), false);
        full_print(&cem);
    }

    #[test]    
    fn test_cem_06() {
        test_cem_06_helper(1);
        
        test_cem_06_helper(2);
        test_cem_06_helper(3);
        test_cem_06_helper(4);
        test_cem_06_helper(5);

        test_cem_06_helper(0);
    }

    #[test]    
    fn test_cem_07() {
        let check_only_first = |x: &CemInner| {
            assert_eq!(x.get_value(), Some(0x11));
            assert_eq!(x.print_om(0), "|F1F0|F0F0|...");
            assert_eq!(x.print_mm(0), "|11|00|...");
            assert_eq!(x.print_am(0), "|00|...");
            assert_eq!(x.trig_cur_am, false);
            assert_eq!(x.error(), false);
            assert_eq!(x.am_pos, None);
            assert_eq!(x.am_last_clear_pos, None);
            assert_eq!(x.om_am_pos, 0);
            assert_eq!(x.om_mm_pos, 0);
            assert_eq!(x.om_am_end_pos, 0);
            assert_eq!(x.om_mm_end_pos, 0);
        };

        let mut cem = CemInner::new(1024, 1024);
        cem.set_value(0x11);
        check_only_first(&cem);

        // M : main mem cell
        // A : addi clear mem cell
        // D : addi dirty mem cell

        //  del        here
        //   ↓    then  ↓
        // M A          M
        cem.create_cell();
        assert_eq!(cem.print_om(0), "|F1F1|F0F0|...");
        assert_eq!(cem.print_mm(0), "|11|00|...");
        assert_eq!(cem.print_am(0), "|00|00|...");
        assert_eq!(cem.trig_cur_am, true);
        cem.delete_cell();
        check_only_first(&cem);

        
        //  del        here
        //   ↓    then  ↓
        // M D          M
        cem.create_cell();
        cem.set_value(0x33);
        cem.delete_cell();
        check_only_first(&cem);

        
        //  del         {here+del}       here
        //   ↓     then     ↓       then  ↓
        // M D A          M A             M
        cem.create_cell();
        cem.create_cell();
        cem.prev_cell();
        cem.set_value(0x33);
        assert_eq!(cem.print_am(0), "|33|00|00|...");
        cem.delete_cell();
        assert_eq!(cem.get_value(), Some(0x00));
        cem.delete_cell();
        check_only_first(&cem);

        //    del         {here+del}         here
        //     ↓     then     ↓         then  ↓
        // M D D A          M D A             M
        cem.create_cell();
        cem.create_cell();
        cem.prev_cell();
        cem.create_cell();
        cem.set_value(0x44);
        cem.prev_cell();
        cem.set_value(0x33);
        assert_eq!(cem.print_am(0), "|33|44|00|00|...");
        cem.next_cell();
        cem.delete_cell();
        assert_eq!(cem.get_value(), Some(0x33));
        assert_eq!(cem.print_am(0), "|33|00|00|...");
        cem.delete_cell();
        assert_eq!(cem.get_value(), Some(0x00));
        assert_eq!(cem.print_am(0), "|00|00|...");
        cem.delete_cell();
        check_only_first(&cem);


        //        del            {here+del}       {here+del}      here
        //         ↓      then        ↓     then      ↓      ...   ↓ 
        // M D A A A A          M D A A A         M D A A          M
        cem.create_cell();  //D
        cem.set_value(0x33);
        cem.create_cell();  //A[1]
        cem.create_cell();  //A[2]
        cem.create_cell();  //A[3]
        cem.create_cell();  //A[4]
        cem.prev_cell();    //cur : A[3]
        assert_eq!(cem.am_pos, Some(3));
        assert_eq!(cem.print_am(0), "|33|00|00|00|00|00|...");
        assert_eq!(cem.print_om(0), "|F1F5|F0F0|...");
        cem.delete_cell();
        assert_eq!(cem.print_am(0), "|33|00|00|00|00|...");
        assert_eq!(cem.am_pos, Some(2));
        assert_eq!(cem.get_value(), Some(0x00));
        cem.delete_cell();
        assert_eq!(cem.print_am(0), "|33|00|00|00|...");
        assert_eq!(cem.am_pos, Some(1));
        assert_eq!(cem.get_value(), Some(0x00));
        cem.delete_cell();
        assert_eq!(cem.print_am(0), "|33|00|00|...");
        assert_eq!(cem.am_pos, Some(0));
        assert_eq!(cem.get_value(), Some(0x33));
        cem.delete_cell();
        assert_eq!(cem.print_am(0), "|00|00|...");
        assert_eq!(cem.am_pos, Some(0));
        assert_eq!(cem.get_value(), Some(0x00));
        cem.delete_cell();
        check_only_first(&cem);


        //          del                     
        //           ↓                  
        //      M DAAAA AAAAA AAAAA A   
        //then   
        //                     del            
        //                      ↓        
        //      M DAAAA DAAAA DAAAA    
        //then   
        // ...
        for ind in 0..16 {
            cem.create_cell();
            if ind == 0 { cem.set_value(5); }
        }
        assert_eq!(cem.print_om(0), "|F1T7|F0T7|F0F2|F0F0|...");
        assert_eq!(cem.print_mm(0), "|11|00|...");
        assert_eq!(cem.print_am(0), "|05|00|00|00|00|00|00|00|00|00|00|00|00|00|00|00|00|...");
        for _ in 0..12 { cem.prev_cell(); }
        assert_eq!(cem.am_pos, Some(3));
        cem.delete_cell();
        assert_eq!(cem.am_pos, Some(2));
        assert_eq!(cem.print_om(0), "|F1T7|F0T7|F0F1|F0F0|...");
        assert_eq!(cem.print_mm(0), "|11|00|...");
        assert_eq!(cem.print_am(0), "|05|00|00|00|00|00|00|00|00|00|00|00|00|00|00|00|...");

        cem.next_cell();
        cem.next_cell();
        cem.create_cell();
        cem.set_value(0x0A);
        for _ in 0..5 { cem.next_cell(); }
        cem.set_value(0x0F);
        cem.next_cell();
        cem.delete_cell();
        cem.next_cell();
        cem.next_cell();
        assert_eq!(cem.print_om(0), "|F1T7|F0T7|F0F1|F0F0|...");
        assert_eq!(cem.print_mm(0), "|11|00|...");
        assert_eq!(cem.print_am(0), "|05|00|00|00|00|0A|00|00|00|00|0F|00|00|00|00|00|...");
        assert_eq!(cem.am_pos, Some(12));
        assert_eq!(cem.am_last_clear_pos, Some(10));

        cem.delete_cell();
        assert_eq!(cem.am_pos, Some(11));
        assert_eq!(cem.am_last_clear_pos, Some(10));
        cem.delete_cell();
        assert_eq!(cem.am_pos, Some(10));
        assert_eq!(cem.am_last_clear_pos, Some(10));
        assert_eq!(cem.get_value(), Some(0x0F));
        for ind in 0..10 { 
            cem.delete_cell(); 
            assert_eq!(cem.am_last_clear_pos, Some(9 - ind));
            assert_eq!(cem.am_pos, Some(9 - ind));
        }
        assert_eq!(cem.print_om(0), "|F1F3|F0F0|...");
        assert_eq!(cem.print_am(0), "|05|00|00|00|...");
        cem.delete_cell();
        assert_eq!(cem.am_pos, Some(0));
        for _ in 0..2 { cem.delete_cell(); }
        check_only_first(&cem);
    }

    #[test]
    fn test_cem_08() {
        let move_to_start = |cem: &mut CemInner| {
            loop { 
                cem.prev_cell();
                if cem.get_value() == Some(0x01) { break; }  
            }
        };

        let move_forward = |cem: &mut CemInner, step: usize| {
            for _ in 0..step { cem.next_cell(); }
        };

        let move_forward_while = |cem: &mut CemInner, value: u8| {
            loop { 
                cem.next_cell(); 
                if cem.get_value() == Some(value) { break; }
            }
        };

        let mm_const = "|01|02|03|04|05|06|07|08|09|0A|0B|0C|0D|0E|0F|10|11|12|13|14|15|16|17|18|00|...";
        let om_const = "|T7F0|T7F0|T7F0|F3F0|F0F0|...";
        let mut cem = CemInner::new(1024, 1024);
        cem.set_value(0x01);
        for ind in 2..25 {
            cem.next_cell();
            cem.set_value(ind);
        }
        assert_eq!(cem.print_om(0), om_const);
        assert_eq!(cem.print_mm(0), mm_const);

        move_to_start(&mut cem);
        move_forward(&mut cem, 7);
        assert_eq!(cem.get_value(), Some(0x08));
        cem.create_cell();
        assert_eq!(cem.print_om(0), "|T7F1|F1F0|T7F0|T7F0|F2F0|F0F0|...");
        assert_eq!(cem.print_mm(0), mm_const);
        cem.delete_cell();
        assert_eq!(cem.print_om(0), om_const);
        assert_eq!(cem.print_mm(0), mm_const);

        move_forward_while(&mut cem, 15);
        cem.create_cell();
        assert_eq!(cem.print_om(0), "|T7F1|T7F0|F1F0|T7F0|F2F0|F0F0|...");
        assert_eq!(cem.print_mm(0), mm_const);
        cem.delete_cell();
        assert_eq!(cem.print_om(0), om_const);
        assert_eq!(cem.print_mm(0), mm_const);


        move_forward_while(&mut cem, 20);
        cem.create_cell();
        assert_eq!(cem.print_om(0), "|T7F1|T7F0|F6F0|F4F0|F0F0|...");
        assert_eq!(cem.print_mm(0), mm_const);
        cem.delete_cell();
        assert_eq!(cem.print_om(0), om_const);
        assert_eq!(cem.print_mm(0), mm_const);

        move_forward_while(&mut cem, 21);
        cem.create_cell();
        assert_eq!(cem.print_om(0), "|T7F1|T7F0|F7F0|F3F0|F0F0|...");
        assert_eq!(cem.print_mm(0), mm_const);
        cem.delete_cell();
        assert_eq!(cem.print_om(0), om_const);
        assert_eq!(cem.print_mm(0), mm_const);

        move_forward_while(&mut cem, 22);
        cem.create_cell();
        assert_eq!(cem.print_om(0), "|T7F1|T7F0|T7F0|F1F0|F2F0|F0F0|...");
        assert_eq!(cem.print_mm(0), mm_const);
        cem.delete_cell();
        assert_eq!(cem.print_om(0), om_const);
        assert_eq!(cem.print_mm(0), mm_const);

        assert_eq!(cem.error(), false);
    }
    
    #[test]
    fn test_cem_09() {
        let mut cem = CemInner::new(1024, 1024);
        for _ in 0..15 { cem.next_cell(); }
        assert_eq!(cem.print_om(0), "|T7F0|T7F0|F2F0|F0F0|...");
        assert_eq!(cem.print_mm(0), "|00|00|00|00|00|00|00|00|00|00|00|00|00|00|00|00|00|...");
        for _ in 0..15 { cem.prev_cell(); }
        assert_eq!(cem.print_om(0), "|F1F0|F0F0|...");
        assert_eq!(cem.print_mm(0), "|00|00|...");

        assert_eq!(cem.error, false);
    }
}