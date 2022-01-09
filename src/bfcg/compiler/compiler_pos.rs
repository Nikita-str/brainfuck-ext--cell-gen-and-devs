
#[derive(Clone)]
pub struct CompilerPos{
    pub line: usize,
    pub symb: usize,

    pub file_name: Option<String>,
}

impl CompilerPos{
    pub fn new_wo_name() -> Self { Self{ line: 1, symb: 1, file_name: None } }
    pub fn new(file_name: String) -> Self { Self{ line: 1, symb: 1, file_name: Some(file_name) } }

    pub fn maybe_add_char(&mut self, c: Option<char>) {
        match c {
            None => {},
            Some(c) => self.add_char(c),
        }
    }

    pub fn add_char(&mut self, c: char) {
        if c == super::compiler::NEXT_LINE { self.next_line(); }
        else { self.symb += 1; }
    }

    pub fn next_line(&mut self) {
        self.line += 1; 
        self.symb = 1;
    }
    
    pub fn next_symb(&mut self) {
        self.symb += 1;
    }
}