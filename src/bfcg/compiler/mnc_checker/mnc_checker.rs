
pub trait MacroNameCodeChecker{
    fn check(&self, name: &str, code: &str) -> Option<ChekerErrorMNC>;
}

pub struct ChekerErrorMNC { pub error: String }

impl ToString for ChekerErrorMNC {
    fn to_string(&self) -> String { format!("{}", self.error) }
}