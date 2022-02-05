use std::{collections::LinkedList, io::Write};


pub(in super) 
struct ConsoleInner {
    buffer: String,
    last_line: LinkedList<char>,
    error: bool,
    
}

impl ConsoleInner{
    pub fn new() -> Self{
        Self {
            buffer: String::with_capacity(0x100),
            last_line: LinkedList::new(),
            error: false,
        }
    }

    pub fn read_char(&mut self) -> Option<char> {
        if self.error() { return None }
        
        if self.last_line.is_empty() { 
            if let Err(_) = std::io::stdin().read_line(&mut self.buffer) {
                self.error = true;
                return None
            } else {
                for c in self.buffer.chars() { self.last_line.push_back(c); }
                self.last_line.push_back('\n');
                self.buffer.clear();
            }
        }

        if let Some(x) = self.last_line.pop_front() {
            return Some(x)
        } else {
            panic!("[ALGO ERROR]")
        }
    }

    pub fn write_char(&mut self, write: char) {
        if self.error() { return }

        print!("{}", write);
        if let Err(_) = std::io::stdout().flush() { self.error = true; }
    }

    pub fn error(&self) -> bool { self.error }
}

