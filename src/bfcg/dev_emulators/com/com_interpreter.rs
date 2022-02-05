use std::collections::HashMap;

use crate::bfcg::compiler::valid_cmd::ValidCMD;

pub struct ComInterpreter {
    cur_pos: Option<usize>,
    mem: Vec<ValidCMD>,
    
    cur_open_jmp: Vec<usize>,
    open_to_close: HashMap<usize, usize>,

    await_jmp_byte: Option<ValidCMD>, // only None | Some({Start|End}WhileNZ)

    error: bool,
    /// has no effect on internal behavior
    infinity: bool,
}

impl ComInterpreter {

    fn o2c_init(mem: &Vec<ValidCMD>) -> Option<HashMap<usize, usize>> {
        let mut ret = HashMap::new();
        let mut open_pos = vec![];
        for (pos, x) in mem.into_iter().enumerate() {
            match x {
                ValidCMD::StartWhileNZ => { open_pos.push(pos); }
                ValidCMD::EndWhileNZ => {
                    if let Some(start_pos) = open_pos.pop() {
                        ret.insert(start_pos, pos);
                    } else {
                        return None
                    }
                }
                _ => {}
            }
        }
        if !open_pos.is_empty() { return None }
        Some(ret)
    }

    pub fn new(mem: Vec<ValidCMD>) -> Self {
        let open_to_close = Self::o2c_init(&mem);

        let (open_to_close, error) = 
        if let Some(x) = open_to_close { (x, false) }
        else { (HashMap::new(), true) };
        
        Self {
            cur_pos: None,
            mem,

            cur_open_jmp: vec![],
            open_to_close,
            
            await_jmp_byte: None,

            error,
            infinity: false,
        }
    }

    /// just get cur cmd, without doing COM operation (JMP)
    pub fn get_cur_cmd(&mut self) -> Option<ValidCMD> {
        let pos = if let Some(x) = self.cur_pos { x } else { self.error = true; 0 };
        if self.error() { return None }
        Some(self.mem.get(pos).unwrap().clone())
    }

    pub fn move_forward(&mut self) {
        if self.error() { return }
        let pos = if let Some(x) = self.cur_pos { x + 1 } else { 0 };
        if pos >= self.mem.len() { self.error = true }
        self.cur_pos = Some(pos);
    }

    /// do COM operation if need + get cur cmd
    pub fn com_get_cmd(&mut self) -> Option<ValidCMD> {
        if self.await_jmp_byte.is_some() { self.error = true; return None }

        self.move_forward();
        let cmd = self.get_cur_cmd();
        let cmd = if let Some(x) = cmd { x } else { return cmd };
        
        if let ValidCMD::StartWhileNZ = cmd { 
            self.cur_open_jmp.push(self.cur_pos.unwrap());
            self.await_jmp_byte = Some(cmd.clone());
        }
        if let ValidCMD::EndWhileNZ = cmd { 
            self.await_jmp_byte = Some(cmd.clone()); 
        }

        Some(cmd)
    }

    /// made COM `JUMP` if need, else - cur state set in error
    /// #### ! can infinity
    /// if `byte != 0` and loop is `[]`
    pub fn com_jmp_byte(&mut self, byte: usize) {
        if self.await_jmp_byte.is_none() { self.error = true; return }
        if self.error() { return }

        let jmp_type = self.await_jmp_byte.clone().unwrap();
        match jmp_type {
            ValidCMD::StartWhileNZ => { 
                if byte == 0 {  
                    let start_pos = self.cur_open_jmp.pop().unwrap();
                    if let Some(pos) = self.open_to_close.get(&start_pos) {
                        self.cur_pos = Some(*pos);
                    } else { panic!("[ALGO ERROR] : bad o2c init") }
                }
            }
            ValidCMD::EndWhileNZ => {
                let start_pos = self.cur_open_jmp.pop().unwrap(); // if byte == 0 => just close
                if byte != 0 {
                    if self.cur_pos == Some(start_pos + 1) { self.infinity = true; } 
                    self.cur_pos = Some(start_pos); 
                }
            }
            _ => { panic!("[ALGO ERROR]") }
        }

        self.await_jmp_byte = None;
    }

    pub fn infinity(&self) -> bool { return self.infinity }
    pub fn error(&self) -> bool { return self.error }

    pub fn print(&self) -> String {
        let mut ret = String::with_capacity(0x100);
        ret += "MEM: _";
        for cmd in &self.mem { ret.push(cmd.clone().std_to_char()) }
        ret += "\n";
        ret += "MEM: ";
        if let Some(x) = self.cur_pos {  
            for _ in 0..=x { ret.push(' '); }
        }
        ret += "↑";
        return ret
    }
}