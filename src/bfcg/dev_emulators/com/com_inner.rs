
pub(in super)
struct ComInner{
    mem_size: usize,
    mem: Vec<u8>,
    cur_pos: usize,
    error: bool,
}

impl ComInner{
    pub fn new(mem_size: usize) -> Self{
        let mut mem = Vec::with_capacity(std::cmp::min(0x100, mem_size));
        mem.push(0x00);

        Self{ 
            mem_size,
            mem,
            cur_pos: 0,
            error: mem_size == 0,
        }
    }

    pub fn error(&self) -> bool { self.error }

    pub fn get_cur_cell(&mut self) -> Option<u8> { 
        if self.error { return None }
        Some(self.mem[self.cur_pos]) 
    }

    pub fn set_cur_cell(&mut self, x: u8) { 
        if self.error { return }
        self.mem[self.cur_pos] = x; 
    }

    pub fn init<'a, Iter: Iterator<Item = &'a u8>>(&mut self, iter: Iter) {
        for x in iter {
            if self.error() || self.mem.len() == self.mem_size { self.error = true; return }
            self.set_cur_cell(*x);
            self.move_forward();
        }
    }

    pub fn move_backward(&mut self) {
        if self.error { return }
        if self.cur_pos == 0 {  self.error = true; return }
        self.cur_pos -= 1;
    }

    pub fn move_forward(&mut self) {
        if self.error { return }
        self.cur_pos += 1;
        if self.cur_pos == self.mem_size { self.error = true; return }
        if self.mem.len() == self.cur_pos { self.mem.push(0x00); }
    }

    /// ~ move forward jmp_len times 
    pub fn jump_forward(&mut self, jmp_len: usize) {
        if self.error { return }
        self.cur_pos += jmp_len;
        if self.cur_pos >= self.mem_size { self.error = true; return } 
        if self.mem.len() <= self.cur_pos {
            self.mem.resize(self.cur_pos, 0x00);
        }
    }

    /// ~ move backward jmp_len times 
    pub fn jump_backward(&mut self, jmp_len: usize) {
        if self.error { return }
        if jmp_len > self.cur_pos { self.error = true; return }
        self.cur_pos -= jmp_len;
    }
}

