
// MAYBE: we can use (consecutive-xx-amount == 0xF) instead overflow-xx-flag


// main, additional & order can be on the same memory device, 
//       just in different pos, like this:
//               |   main   | additional | order |
struct CemInner{
    main_mem: Vec<u8>,
    mm_size: usize,
    mm_pos: usize,

    additional_mem: Vec<u8>,
    am_size: usize,
    am_pos: usize,

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

    reg_mm: usize, // main_memory pos in om
    reg_am: usize, // addi_memory pos in om
    reg_cur_am: bool, // if 0 => mm else am

    reg_con_amount: u8, // u4 is enough
    reg_stay_on_end: bool, 

    /// for emulate invalid operation  
    error: bool,
}

const MM_OF_SHIFT: usize = 7;
const AM_OF_SHIFT: usize = 3;
const MM_AMOUNT_SHIFT: usize = 4;
const AM_AMOUNT_SHIFT: usize = 0;

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
            am_pos: 0,

            order_mem: om,
            om_size,

            reg_mm: 0,
            reg_am: 0,
            reg_cur_am: false, // first mem-cell always is mm even in run-time

            reg_con_amount: 1,
            reg_stay_on_end: true,

            error: false,
        }
    }

    pub fn error(&self) -> bool { self.error }

    pub fn next_cell(&mut self) {
        if self.error { return }

        if self.reg_stay_on_end {
            
        } else {

        }
    }


    pub fn print_om(&self, new_line_each: usize) -> String {
        let mut ret = String::new();
        for (ind, x) in (&self.order_mem).into_iter().enumerate() {
            ret += &format!(
                "|{}{}{}{}|", 
                if (x >> MM_OF_SHIFT) & 1 == 1 { 'T' } else {'F'},
                (x >> MM_AMOUNT_SHIFT) & 0x7, 
                if (x >> AM_OF_SHIFT) & 1 == 1 { 'T' } else {'F'},
                (x >> AM_AMOUNT_SHIFT) & 0x7, 
            );
            if (new_line_each > 0) && ind != 0 && (ind % new_line_each == 0) { ret += "\n"; } 
        }
        ret
    } 
}