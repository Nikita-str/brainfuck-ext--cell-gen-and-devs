

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerPos{
    pub line: usize,
    pub symb: usize,

    pub blocked_pos: usize,
}

impl CompilerPos{
    pub fn new() -> Self { Self{ line: 1, symb: 0, blocked_pos: 0 } }

    pub fn maybe_add_char(&mut self, c: Option<char>) {
        match c {
            None => {},
            Some(c) => self.add_char(c),
        }
    }

    fn is_blocked_with_one_char_unblock(&mut self) -> bool {
        if self.blocked_pos == 0 { return false }
        self.blocked_pos -= 1;
        true
    }

    pub fn add_char(&mut self, c: char) {
        if self.is_blocked_with_one_char_unblock() { return }
        if c == super::compiler::NEXT_LINE { self.next_line(); }
        else { self.symb += 1; }
    }

    pub fn next_line(&mut self) {
        if self.is_blocked_with_one_char_unblock() { return }
        self.line += 1; 
        self.symb = 0;
    }
    
    pub fn next_symb(&mut self) {
        if self.is_blocked_with_one_char_unblock() { return }
        self.symb += 1;
    }

    pub fn pos_back(&mut self) {
        self.blocked_pos += 1;
    }
}


pub struct ExtCompilerPos{
    pub pos: Option<CompilerPos>,
    pub file_name: String,
}


impl ToString for CompilerPos {
    fn to_string(&self) -> String { format!("[line: {}  symb: {}]", self.line, self.symb) }
} 

impl ToString for ExtCompilerPos {
    fn to_string(&self) -> String { 
        if let Some(x) = &self.pos { format!("<in file: {} at position: {}>",self.file_name, x.to_string()) }
        else { format!("<in file: {} at unknown position>", self.file_name) }
    }
}