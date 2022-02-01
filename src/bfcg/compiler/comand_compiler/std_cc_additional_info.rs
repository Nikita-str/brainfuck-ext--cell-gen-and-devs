use boolvec::BoolVec;

const PREPARED_PRS: usize = super::std_cc::MAX_PR - 1; // -1 cause USER_PR is not prepared

#[derive(PartialEq, Eq)]
pub(in crate::bfcg::compiler::comand_compiler)
enum PrPrepared {
    Console,
    Win,
    MemCell,
    MemCmd,
}
impl PrPrepared { 
    pub fn to_index(&self) -> usize { 
        match self{
            PrPrepared::Console => 0,
            PrPrepared::Win => 1,
            PrPrepared::MemCell => 2,
            PrPrepared::MemCmd => 3,
        }
    }

    pub fn from_name(name: &str) -> Option<Self>{
        match name {
            "console" => Some(Self::Console),
            "win" | "screen" | "display" => Some(Self::Win),
            //"cell" | "cem" => Some(Self::MemCell),
            //"cmd" | "com" => Some(Self::MemCmd),
            _ => None,
        }
    }
 }

pub(in crate::bfcg::compiler::comand_compiler)
struct StdCcAditionalInfo {
    one_pr_reserve_sz: usize,
    jump_pass_amount: usize,
    pr_prepared: BoolVec,
}

impl StdCcAditionalInfo{
    pub fn new() -> Self{
        let mut pr_prepared = BoolVec::new();
        for _ in 0..PREPARED_PRS { pr_prepared.push(false); }

        let mut ret = Self {
            one_pr_reserve_sz: 0,
            jump_pass_amount: 0,
            pr_prepared,
        };
        ret.set_prepared(PrPrepared::MemCell);
        ret.set_prepared(PrPrepared::MemCmd);
        ret
    }

    pub fn set_prepared(&mut self, prepared: PrPrepared) {
        self.pr_prepared.set(prepared.to_index(), true);
    }

    pub fn set_pr_reserve_sz(&mut self, sz: usize) { self.one_pr_reserve_sz = sz }
    pub fn get_pr_reserve_sz(&self) -> usize { self.one_pr_reserve_sz }

    pub fn set_jump_pass_amount(&mut self, jump_pass_amount: usize) { self.jump_pass_amount = jump_pass_amount }
    pub fn get_jump_pass_amount(&self) -> usize { self.jump_pass_amount }

    pub fn is_all_prepared(&self) -> bool {
        for x in 0..PREPARED_PRS{
            if !self.pr_prepared.get(x).unwrap() { return false }
        }
        return true
    }
}

impl Default for StdCcAditionalInfo{
    fn default() -> Self {
        Self { 
            one_pr_reserve_sz: 0,
            jump_pass_amount: 0,
            pr_prepared: BoolVec::new() 
        }
    }
}