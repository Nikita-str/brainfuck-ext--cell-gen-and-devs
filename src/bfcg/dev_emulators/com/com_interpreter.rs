use crate::bfcg::compiler::valid_cmd::ValidCMD;


struct JmpPosPair{
    pub start: usize,
    pub end: usize,
}

pub struct ComInterpreter {
    cur_pos: usize,
    mem: Vec<ValidCMD>,
    jmp_pos: Vec<JmpPosPair>,
    error: bool,
}

impl ComInterpreter {

    fn jmp_pos_init(mem: &Vec<ValidCMD>) -> Option<Vec<JmpPosPair>> {
        let mut ret = vec![];

        let mut open_pos = vec![];
        for (pos, x) in mem.into_iter().enumerate() {
            match x {
                ValidCMD::StartWhileNZ => {
                    open_pos.push(ret.len());
                    ret.push(JmpPosPair{ start: pos, end: 0 });
                }
                ValidCMD::EndWhileNZ => {
                    if let Some(ind) = open_pos.pop() {
                        ret[ind].end = pos;
                    } else {
                        return None
                    }
                }
                _ => {}
            }
        }

        Some(ret)
    }

    pub fn new(mem: Vec<ValidCMD>) -> Self {
        let jmp_pos = Self::jmp_pos_init(&mem);
        let error = jmp_pos.is_none();

        Self {
            cur_pos: 0,
            mem,
            jmp_pos: if let Some(x) = jmp_pos { x} else { vec![] },
            error,
        }
    }

    pub fn get_cmd(&mut self) -> Option<ValidCMD> {
        if self.cur_pos >= self.mem.len() { self.error = true }
        if self.error { return None }
        Some(self.mem.get(self.cur_pos).unwrap().clone())
    }


}