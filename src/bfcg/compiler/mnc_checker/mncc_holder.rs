use std::collections::HashMap;
use super::{MacroNameCodeChecker, ChekerErrorMNC};


pub struct HolderChekerMNC<'a>{
    checkers: HashMap<&'a str, &'a dyn MacroNameCodeChecker> 
}

impl<'a> HolderChekerMNC<'a>{
    pub fn new() -> Self { Self{ checkers: HashMap::new() } }

    pub fn add_checker(&mut self, name: &'a str, checker: &'a impl MacroNameCodeChecker) {
        if let Some(_) = self.checkers.insert(name, checker) { 
            panic!("checker with that name already exist (just use other)") 
        }
    }

    /// ### WARNING!
    /// macro_code must not be transformed yet  
    pub fn check_all(&self, macro_name: &str, macro_code: &str) -> Option<(String, ChekerErrorMNC)> {
        for (name, checker) in &self.checkers {
            if let Some(error) = checker.check(macro_name, macro_code) {
                return Some((name.to_string(), error))
            }
        }

        None
    }
}