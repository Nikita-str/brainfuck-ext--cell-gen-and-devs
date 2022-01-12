

pub enum SettingActionResultType{
    /// other rule
    NoSatisfy,
    /// highly probably right rule but wrong syntax
    Error{ error: String },
    /// right rule, but warning
    OkWithWarning{ warning: String },
    /// right rule
    Ok,
}

impl SettingActionResultType{
    pub fn new_ok() -> Self { Self::Ok }
    pub fn new_no() -> Self { Self::NoSatisfy }
    pub fn new_warning_s(warning: String) -> Self { Self::OkWithWarning{ warning } }
    pub fn new_warning(warning: &str) -> Self { Self::OkWithWarning{ warning: warning.to_owned() } }
    pub fn new_error_s(error: String) -> Self { Self::Error{ error } }
    pub fn new_error(error: &str) -> Self { Self::Error{ error: error.to_owned() } }

    pub fn is_ok(&self) -> bool { if let Self::Ok = self { true } else { false } } 
    pub fn is_no(&self) -> bool { if let Self::NoSatisfy = self { true } else { false } } 
    pub fn is_with_warning(&self) -> bool { if let Self::OkWithWarning{..} = self { true } else { false } } 
    pub fn is_error(&self) -> bool { if let Self::Error{..} = self { true } else { false } } 

    pub fn is_right_rule(&self) -> bool { self.is_ok() || self.is_with_warning() }

    pub fn get_warining(self) -> Option<String> { 
        if let Self::OkWithWarning{warning} = self { Some(warning) }
        else { None }
    }
}

pub struct SettingActionResult{
    pub result_type: SettingActionResultType,
    pub parent_must_process: bool, 
}


macro_rules! sar_proxy_1p {
    ( $fn_name:ident, $p_type:ty ) => {
        pub fn $fn_name(x: $p_type) -> Self { Self::new(SettingActionResultType:: $fn_name (x), false) }
    };
}
macro_rules! sar_proxy_parent {
    ( $fn_name:ident, $p_type:ty ) => {
        pub fn $fn_name(x: $p_type, parent_must_process: bool) -> Self { Self::new(SettingActionResultType:: $fn_name (x), parent_must_process) }
    };
}

macro_rules! sar_proxy_self_to_bool {
    ( $fn_name:ident ) => {
        pub fn $fn_name(&self) -> bool { self.result_type. $fn_name () }
    };
}

impl SettingActionResult{
    pub fn new_no() -> Self { Self::new(SettingActionResultType::NoSatisfy, false) }
    pub fn new_ok(parent_must_process: bool) -> Self { Self::new(SettingActionResultType::Ok, parent_must_process) }

    pub fn new_ok_need_in_parent() -> Self { Self::new(SettingActionResultType::new_ok(), true) }
    /* 
    pub fn new_warning(warning: &str) -> Self { 
        Self::new(SettingActionResultType::new_warning(warning), true) 
    }
    pub fn new_warning_s_need_in_parent(warning: String) -> Self { 
        Self::new(SettingActionResultType::new_warning_s(warning), true) 
    }
    */

    sar_proxy_1p!(new_error_s, String);
    sar_proxy_1p!(new_error, &str);
    sar_proxy_parent!(new_warning_s, String);
    sar_proxy_parent!(new_warning, &str);
    sar_proxy_self_to_bool!(is_right_rule);
    sar_proxy_self_to_bool!(is_error);

    pub fn new(result_type: SettingActionResultType, parent_must_process: bool) -> Self {
        Self{ result_type, parent_must_process }
    }
    pub fn parent_must_process(&self) -> bool { self.parent_must_process }
    pub fn into_result(self) -> SettingActionResultType { self.result_type }

    
    pub fn get_warining(self) -> Option<String> { self.result_type.get_warining() } 
}