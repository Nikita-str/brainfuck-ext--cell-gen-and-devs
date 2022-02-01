use boolvec::BoolVec;

const PREPARED_PRS: usize = super::std_dir_mem_cc::MAX_PR - 1; // -1 cause USER_PR is not prepared

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
            "cell" | "cem" => Some(Self::MemCell),
            "cmd" | "com" => Some(Self::MemCmd),
            _ => None,
        }
    }
 }

pub(in crate::bfcg::compiler::comand_compiler)
struct SDMCCAditionalInfo {
    one_pr_reserve_sz: usize,
    pr_prepared: BoolVec,
}

impl SDMCCAditionalInfo{
    pub fn new() -> Self{
        let mut pr_prepared = BoolVec::new();
        for _ in 0..PREPARED_PRS { pr_prepared.push(false); }
        Self {
            one_pr_reserve_sz: 0,
            pr_prepared,
        }
    }

    pub fn set_prepared(&mut self, prepared: PrPrepared) {
        self.pr_prepared.set(prepared.to_index(), true);
    }

    pub fn set_pr_reserve_sz(&mut self, sz: usize) { self.one_pr_reserve_sz = sz }
    pub fn get_pr_reserve_sz(&self) -> usize { self.one_pr_reserve_sz }

    pub fn is_all_prepared(&self) -> bool {
        for x in 0..PREPARED_PRS{
            if !self.pr_prepared.get(x).unwrap() { return false }
        }
        return true
    }
}

impl Default for SDMCCAditionalInfo{
    fn default() -> Self {
        Self { 
            one_pr_reserve_sz: 0, 
            pr_prepared: BoolVec::new() 
        }
    }
}