use std::collections::HashMap;


pub struct MemInit{
    mem_init: HashMap<usize, Vec<u8>>
}

impl MemInit{
    pub fn new() -> Self { 
        Self { mem_init: HashMap::new() }
    }

    pub fn is_empty(&self) -> bool { self.mem_init.is_empty() }
    pub fn len(&self) -> usize { self.mem_init.len() }

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
}