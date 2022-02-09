use std::{collections::LinkedList, io::Write};


pub(in super) 
struct ConsoleInner {
    buffer: String,
    last_line: LinkedList<char>,
    error: bool,
    
    need_write_state: bool,
    prev_state: Option<ConsoleState>,
}

impl ConsoleInner{
    pub fn new(need_write_state: PrivateConsoleNeedWrite) -> Self{
        Self {
            buffer: String::with_capacity(0x100),
            last_line: LinkedList::new(),
            error: false,

            need_write_state: need_write_state.need_write_state,
            prev_state: None,
        }
    }

    pub fn need_write_state(&self) -> bool { self.need_write_state }

    pub fn read_char(&mut self) -> Option<char> {
        if self.error() { return None }
        
        if self.need_write_state {
            if self.prev_state.is_none() { print!("[INPUT ]:\n") } 
            else if self.prev_state.as_ref().unwrap().is_write() { print!("\n[INPUT ]:\n"); }
            self.prev_state = Some(ConsoleState::Read);
            if let Err(_) = std::io::stdout().flush() { self.error = true; }
            if self.error() { return None }
        }

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

        if self.need_write_state {
            if self.prev_state.is_none() { print!("[OUTPUT]:\n") } 
            else if self.prev_state.as_ref().unwrap().is_read() { print!("[OUTPUT]:\n"); }
            self.prev_state = Some(ConsoleState::Write);
        }

        print!("{}", write);
        if let Err(_) = std::io::stdout().flush() { self.error = true; }
    }

    pub fn error(&self) -> bool { self.error }
}


enum ConsoleState{
    Read,
    Write,
}

impl ConsoleState {
    fn is_read(&self) -> bool { if let Self::Read = self { true } else { false } }
    fn is_write(&self) -> bool { if let Self::Write = self { true } else { false } }
}


// --------------------------------------------
// [+] NEED WRITE STATE:

pub(in super)
struct PrivateConsoleNeedWrite{ pub need_write_state: bool }

impl std::str::FromStr for PrivateConsoleNeedWrite {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "+" => { Ok(Self{need_write_state: true}) }
            "-" => { Ok(Self{need_write_state: false}) }
            _ => Err(())
        } 
    }
}

pub (in super)
const DEFAULT_NEED_WRITE_STATE: PrivateConsoleNeedWrite = PrivateConsoleNeedWrite{ need_write_state: false};

// [-] NEED WRITE STATE
// --------------------------------------------
