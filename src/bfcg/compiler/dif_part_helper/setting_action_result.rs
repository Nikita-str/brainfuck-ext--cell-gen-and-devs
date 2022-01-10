

pub enum SettingActionResult{
    /// other rule
    NoSatisfy,
    /// highly probably right rule but wrong syntax
    Error{ error: String },
    /// right rule, but warning
    OkWithWarning{ warning: String },
    /// right rule
    Ok,
}

impl SettingActionResult{
    pub fn new_ok() -> Self { Self::Ok }
    pub fn new_no() -> Self { Self::NoSatisfy }
    pub fn new_warning_s(warning: String) -> Self { Self::OkWithWarning{ warning } }
    pub fn new_warning(warning: &str) -> Self { Self::OkWithWarning{ warning: warning.to_owned() } }
    pub fn new_error_s(error: String) -> Self { Self::Error{ error } }
    pub fn new_error(error: &str) -> Self { Self::Error{ error: error.to_owned() } }

    pub fn is_ok(&self) -> bool { if let Self::Ok = self { true } else { false } } 
    pub fn is_no(&self) -> bool { if let Self::NoSatisfy = self { true } else { false } } 
    pub fn is_with_warning(&self) -> bool { if let Self::OkWithWarning{..} = &self { true } else { false } } 
    pub fn is_error(&self) -> bool { if let Self::Error{..} = self { true } else { false } } 

    pub fn is_right_rule(&self) -> bool { self.is_ok() || self.is_with_warning() }
}