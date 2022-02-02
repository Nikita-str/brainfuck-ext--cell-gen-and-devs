
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

    om_mm_pos: usize, // main_memory pos in om, check {*1} but we need two ptr
    om_am_pos: usize, // addi_memory pos in om, check {*1} but we need two ptr
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
        if con_amount_real > self.reg_con_amount { panic!("[ALGO ERROR]") } 
        con_amount_real == self.reg_con_amount
    }

    fn stay_on_the_end(&self) -> bool {
        let am_can_be_end = (self.am_pos.is_none() && self.am_size == 0)
            || (self.am_pos.unwrap() == (self.am_size - 1)) 
            || (((self.order_mem[self.om_am_pos + 1] >> AM_AMOUNT_SHIFT) & AMOUNT_MAX) == 0);
        
        let mm_can_be_end = (self.mm_pos == (self.mm_size - 1)) 
            || (((self.order_mem[self.om_mm_pos + 1] >> MM_AMOUNT_SHIFT) & AMOUNT_MAX) == 0);

        let con_end = self.cur_con_end();

        am_can_be_end && mm_can_be_end && con_end
    }

    fn cur_con_overflow(&self) -> bool {
        if self.reg_cur_am {
            ((self.order_mem[self.om_am_pos] >> AM_OF_SHIFT) & 1) == 1
        } else {
            ((self.order_mem[self.om_mm_pos] >> MM_OF_SHIFT) & 1) == 1
        }
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


    pub fn print_om(&self, new_line_each: usize) -> String {
        let mut ret = String::new();
        for (ind, x) in (&self.order_mem).into_iter().enumerate() {
            ret += &format!(
                "|{}{}{}{}|", 
                if (x >> MM_OF_SHIFT) & 1 == 1 { 'T' } else {'F'},
                (x >> MM_AMOUNT_SHIFT) & AMOUNT_MAX, 
                if (x >> AM_OF_SHIFT) & 1 == 1 { 'T' } else {'F'},
                (x >> AM_AMOUNT_SHIFT) & AMOUNT_MAX, 
            );
            if (new_line_each > 0) && ind != 0 && (ind % new_line_each == 0) { ret += "\n"; } 
        }
        ret
    } 
}