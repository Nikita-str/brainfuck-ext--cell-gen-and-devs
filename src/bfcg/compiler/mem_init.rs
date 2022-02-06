use std::collections::BTreeMap;

//MemInitMergeError
// MMC : MAIN MEMORY CELL

#[derive(Debug)]
pub struct MemInit{
    param_pos_just: Option<usize>,
    param_pos_win: Option<usize>,
    mem_init: BTreeMap<usize, Vec<u8>>,
}

impl MemInit{
    pub fn new() -> Self { 
        Self { 
            param_pos_just: None,
            param_pos_win: None,
            mem_init: BTreeMap::new(), 
        }
    }

    /// ## Result
    /// * if error => Some(..)
    /// * else => None
    pub fn merge(&mut self, other: Self) -> Option<usize> {
        for (mmc, cell_init) in other.mem_init {
            if self.len_in_mmc(mmc) != 0 { return Some(mmc) }
            self.mem_init.insert(mmc, cell_init);
        }
        None
    }

    pub fn add_just_param(&mut self, byte: u8) -> bool {
        if let Some(j_pos) = self.param_pos_just { 
            self.add_byte(j_pos, byte);
            true 
        } else {
            false
        }
    }

    pub fn can_add_just_param(&self) -> bool { self.param_pos_just.is_some() }
    pub fn can_add_win_param(&self) -> bool { self.param_pos_win.is_some() }

    pub fn get_win_pos(&self) -> usize { self.param_pos_win.unwrap() }

    pub fn set_param_pos(&mut self, just_pos: usize, win_pos: usize) -> bool {
        if !self.is_empty() { return false }
        if just_pos == win_pos { return false }
        self.param_pos_just = Some(just_pos);
        self.param_pos_win = Some(win_pos);
        true
    }

    pub fn is_empty(&self) -> bool { self.mem_init.is_empty() }
    pub fn len(&self) -> usize { self.mem_init.len() }

    pub fn len_in_mmc(&self, mmc: usize) -> usize { 
        if let Some(x) = self.mem_init.get(&mmc) { x.len() }
        else { 0 }
    }

    pub fn len_in_win_mmc(&self) -> usize {
        if let Some(w_pos) = self.param_pos_win { self.len_in_mmc(w_pos) } 
        else { panic!("no win mmc") }        
    }


    pub fn get_memory(&self, after_mmc: usize) -> Option<&Vec<u8>> {
        self.mem_init.get(&after_mmc)
    }

    /// ## params: 
    /// * after_mmc = after main memory cell (after first(0), second(1), ...)  
    /// * byte: what byte we must added after all already added by this mmc addr
    /// for example:
    /// ```
    /// use bf_cell_gen::bfcg::compiler::mem_init::MemInit;
    /// let mut x = MemInit::new();
    /// x.add_byte(0, 42);
    /// x.add_byte(0, 13);
    /// assert_eq!(x.get_memory(0), Some(&vec![42u8, 13u8]));
    /// ``` 
    pub fn add_byte(&mut self, after_mmc: usize, byte: u8) {
        if self.mem_init.contains_key(&after_mmc) {
            self.mem_init.get_mut(&after_mmc).unwrap().push(byte);
        } else {
            self.mem_init.insert(after_mmc, vec![byte]);
        }
    }

    pub fn add_bytes<'u8, I: IntoIterator<Item=&'u8 u8>>(&mut self, after_mmc: usize, bytes: I) {
        for byte in bytes.into_iter() {
            self.add_byte(after_mmc, *byte);
        }
    }

    pub fn code_gen(&self) -> String {
        let mut ret = String::new();
        let mut prev_mmc = 0;
        let mut need_back_shift = 0;

        for (mmc, mem_cell) in self.mem_init.iter() {
            let mmc = *mmc;
            debug_assert!(prev_mmc <= mmc, "b-tree-map must be sorted by key");
            if mem_cell.is_empty() { continue }
            
            let delta = mmc - prev_mmc;
            need_back_shift += delta + mem_cell.len();
            ret += &super::code_gen::cgen_shift_cell_ptr(delta as i32);
            for value in mem_cell {
                super::code_gen::add_cgen_cell_create(&mut ret);
                super::code_gen::add_cgen_set_cell_value(&mut ret, *value, false);
            }

            prev_mmc = mmc;
        }

        ret += &super::code_gen::cgen_shift_cell_ptr(-(need_back_shift as i32));
        ret
    }
}