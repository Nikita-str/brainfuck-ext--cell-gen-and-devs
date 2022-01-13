
pub trait MacroNameCodeChecker{
    fn check(&self, name: &str, code: &str) -> Option<ChekerErrorMNC>;
}

pub struct ChekerErrorMNC { pub error: String }