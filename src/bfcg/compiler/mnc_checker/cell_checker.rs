use crate::bfcg::compiler::valid_cmd::ValidCMD;

use super::{MacroNameCodeChecker, ChekerErrorMNC};

/// name must be:
/// * "@[num]x..x" if in cmds x..x created num not deleted cells
/// * other if all created cells deleted  
pub struct CellCheckerCD{ 
    pub rule_out: String,
    pub open_num: String, 
    pub close_num: String, 
}

impl CellCheckerCD {
    pub fn new_std() -> Self {
        Self {
            rule_out: STD_RULE_OUT.to_string(),
            open_num: STD_NUM_OPEN.to_string(),
            close_num: STD_NUM_CLOSE.to_string(),
        }
    }
    
    fn cell_checker_name(&self, name: &str) -> isize {
        let mut cd_must_be = 0;
        let mut sp = name.split(&self.open_num);
        if let Some(x) = sp.next() {
            if x != &self.rule_out { return 0 }
            if let Some(num) = sp.next().map_or(None, |x|x.split(&self.close_num).next()) {
                if let Ok(x) = num.parse() {
                    cd_must_be = x;
                }
            }
        }

        cd_must_be
    }
}
const STD_RULE_OUT: char = '@';
const STD_NUM_OPEN: char = '[';
const STD_NUM_CLOSE: char = ']';

impl MacroNameCodeChecker for CellCheckerCD {
    fn check(&self, name: &str, code: &str) -> Option<ChekerErrorMNC> {
        if name.len() == 0 { return Some(ChekerErrorMNC{ error: "empty name".to_owned()}) }

        // check name started with? @[num]
        let cd_must_be = self.cell_checker_name(name);

        let mut part_is_call = false;
        let mut cd_amount = 0;
        for part in code.split("%"){
            if part_is_call {
                cd_amount += self.cell_checker_name(part);
            } else {
                for c in part.chars() {
                    if c == ValidCMD::CreateCell.std_to_char() { cd_amount += 1 }
                    if c == ValidCMD::CreateCell.std_to_char() { cd_amount -= 1 }
                }
            }

            part_is_call = !part_is_call;
        }

        if cd_must_be == cd_amount {
            None
        } else {
            let error = format!("wrong amount of {{c, d}} cmds; amount(c) - amount(d) must be {} but was {}", cd_must_be, cd_amount);
            Some(ChekerErrorMNC{ error })
        }
    }
}