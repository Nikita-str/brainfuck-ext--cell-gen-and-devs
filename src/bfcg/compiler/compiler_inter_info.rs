
#[derive(Clone, Debug)]
pub struct CompilerInterInfo{
    pub code_started: bool,
}

impl CompilerInterInfo{
    pub fn new() -> Self{
        Self {
            code_started: false,
        }
    }

    pub fn update_with_other(&mut self, other: &Self) {
        if other.code_started && !self.code_started { 
            self.code_started = true;
            panic!("in current version include file cant compile code => error")
        }
        else if !other.code_started && self.code_started { panic!("this cant be true!") }
    }

    pub fn set_code_start(&mut self) { self.code_started = true; }

    pub fn is_code_started(&self) -> bool { self.code_started }
}