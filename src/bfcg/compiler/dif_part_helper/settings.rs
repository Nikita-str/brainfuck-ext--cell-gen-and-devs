

pub struct Setting{ 
    pub params: Vec<SettingOneParam>, 
}

impl Setting{
    pub fn len(&self) -> usize { self.params.len() }
}


pub struct SettingOneParam{
    pub param: String,
    pub additional_params: Vec<String>,
}

impl SettingOneParam{
    fn new(param: String, additional_params: Vec<String>) -> Self { Self { param, additional_params } }
}


pub struct ErrorSetting{
    pub param: String,
    pub error: ErrorSettingHelper,
}

pub enum ErrorSettingHelper{
    ClosedBeforeOpen,    // "not]hehe["        ! ] is wrong
    TwiceAdditional,     // "xx[not][hehe]"    ! [hehe] is wrong
    EmptyParam,          // "[not;hehe]"       ! nothing before [] is wrong

    CharAfterAdditional, // "x[y;z]not-hehe"   ! n is wrong
}


impl Setting{
    const OPEN: char = '[';
    const CLOSE: char = ']';
    const PARAM_SPLITER: [char; 1] = ['|'];

    fn one_param_prepare(param: &str) -> Result<SettingOneParam, ErrorSettingHelper>{
        let mut main_param = String::new();
        let mut additional = vec![];
        let mut cur_add_param = String::new();

        let mut already_open = false;
        let mut closed = false;

        for c in param.chars() {
            if closed { return Err(ErrorSettingHelper::CharAfterAdditional) }

            if !already_open { 
                if c == Self::OPEN {
                    if main_param.is_empty() { return Err(ErrorSettingHelper::EmptyParam) }
                    already_open = true;
                    continue
                } else if c == Self::CLOSE {
                    return Err(ErrorSettingHelper::ClosedBeforeOpen)
                } else {
                    main_param.push(c);
                }
            } else {
                if c == Self::CLOSE { 
                    closed = true;
                    continue  
                } else if c == Self::OPEN {
                    return Err(ErrorSettingHelper::TwiceAdditional)
                } else if Self::PARAM_SPLITER.contains(&c) {
                    if cur_add_param.is_empty() { return Err(ErrorSettingHelper::EmptyParam) }
                    additional.push(cur_add_param);
                    cur_add_param = String::new();
                }  else {
                    cur_add_param.push(c);
                }
            }
        }

        Ok(SettingOneParam::new(main_param, additional))
    }

    pub(in crate::bfcg::compiler) 
    fn prepare_settings(setting: &str) -> Result<Self, ErrorSetting>{
        let split = setting.split(":");
        let mut setting_params = vec![];
        for part in split {
            match Self::one_param_prepare(part) {
                Ok(one_param) => {setting_params.push(one_param) }
                Err(error) => return Err(ErrorSetting{ param: part.to_owned(), error })
            }
        }

    return Ok(Self{ params: setting_params })
    }
}