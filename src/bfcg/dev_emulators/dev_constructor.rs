use std::collections::{HashMap, HashSet};

use super::{dev::Dev, win::win::SpecialWin};


// TODO: string name ---> Dev 


//------------------------------------------------------------------------
// [+] ERR, WARN, OK

pub enum DevCtorErr {
    UnknownName(String),
    BadParamValue{ name: String, bad_value: String },
    Other(String),
}

impl DevCtorErr {
    pub fn is_unknown(&self) -> bool { matches!(self, Self::UnknownName(_)) }
}

impl ToString for DevCtorErr {
    fn to_string(&self) -> String {
        match self {
            Self::UnknownName(x) => format!("unknown name \"{}\"", x), 
            Self::BadParamValue{name, bad_value} => format!("for parameter {} value {} is inappropriate", name, bad_value),
            Self::Other(x) => x.to_owned(),
        }
    }
}

pub enum DevCtorWarn {
    UnusedDevParam(String),
    Other(String),
}

impl ToString for DevCtorWarn {
    fn to_string(&self) -> String {
        match self {
            Self::UnusedDevParam(x) => format!("unused parameter name \"{}\"", x), 
            Self::Other(x) => x.to_owned(), 
        }
    }
}

pub struct DevCtorOk{
    pub dev: Box<dyn Dev>,
    pub warns: Vec<DevCtorWarn>,
}

impl DevCtorOk{
    pub fn new(dev: Box<dyn Dev>, warns:Vec<DevCtorWarn>) -> Self { Self { dev, warns } }
}

// [-] ERR, WARN, OK
//------------------------------------------------------------------------

//------------------------------------------------------------------------
// [+] TRAIT
pub trait DevCtor {
    fn dev_ctor(dev_name_params: &HashMap<String, String>) -> Result<DevCtorOk, DevCtorErr>;
}

pub trait DevWinCtor {
    fn dev_win_ctor(win: &mut SpecialWin, dev_name_params: &HashMap<String, String>) -> Result<DevCtorOk, DevCtorErr>;
}

pub trait SpecialWinCtor {
    fn special_win_ctor(dev_name_params: &HashMap<String, String>) -> Result<SpecialWin, DevCtorErr>;
}

// [-] TRAIT
//------------------------------------------------------------------------

//------------------------------------------------------------------------
// [+] HELPER:

pub struct DevCtorHelper<'a> {
    dev_name_params: &'a HashMap<String, String>,
    used_params: HashSet<&'a str>,
    warns: Vec<DevCtorWarn>,
}

impl<'a> DevCtorHelper<'a>{ 
    pub fn new(dev_name_params: &'a HashMap<String, String>) -> Self { 
        Self {
            dev_name_params,
            used_params: HashSet::new(),
            warns: Vec::new(),
        }
    }

    pub fn add_warn(&mut self, warn: DevCtorWarn) { self.warns.push(warn); }
    pub fn use_param(&mut self, param_name: &'a str) { self.used_params.insert(param_name); }

    pub fn dev_ctor_parse<T: std::str::FromStr>(&mut self, await_name: &'a str, default: T) -> Result<T, DevCtorErr> {
        if let Some(x) = self.dev_name_params.get(await_name) {
            self.used_params.insert(await_name);
            if let Ok(x) = x.parse() { Ok(x) }
            else { 
                return Err(DevCtorErr::BadParamValue{name: await_name.to_owned(), bad_value: x.to_owned()})
            }
        } 
        else { Ok(default) }
    }

    pub fn add_unused_warn(&mut self) {
        if self.dev_name_params.len() > self.used_params.len() {
            for (p_name, _) in self.dev_name_params {
                if !self.used_params.contains(&p_name.as_str()) { 
                    self.warns.push(DevCtorWarn::UnusedDevParam(p_name.to_owned())) 
                }
            }
        }
    }

    pub fn take_warn(self) -> Vec<DevCtorWarn> { self.warns }
}


// [-] HELPER
//------------------------------------------------------------------------


//------------------------------------------------------------------------
// [+] MACROS:

#[macro_export]
macro_rules! dev_ctor_parse_unwrap {
    ($helper:ident, $await_name:expr, $default:ident) => {
        {
            let temp = $helper.dev_ctor_parse($await_name, $default);
            if let Err(x) = temp { return Err(x) }
            if let Ok(x) = temp { x }
            else { panic!("never!") }
        }
    };
}


#[macro_export]
macro_rules! dev_ctor_impl {
    ($dev_type: ty $([$await_name:expr, $default:ident])*) => {
        impl crate::bfcg::dev_emulators::dev_constructor::DevCtor for $dev_type {
            fn dev_ctor(dev_name_params: &std::collections::HashMap<String, String>) 
            -> Result<crate::bfcg::dev_emulators::dev_constructor::DevCtorOk, 
                      crate::bfcg::dev_emulators::dev_constructor::DevCtorErr> 
            {
                let mut helper = crate::bfcg::dev_emulators::dev_constructor::DevCtorHelper::new(dev_name_params);

                let box_value = Box::new(
                    <$dev_type>::new(
                        $( crate::dev_ctor_parse_unwrap!(helper,  $await_name, $default), )*
                    )
                );

                helper.add_unused_warn();
                let warns = helper.take_warn();
        
                Ok(crate::bfcg::dev_emulators::dev_constructor::DevCtorOk::new(box_value, warns))
            }
        }
    };
}

// [-] MACROS
//------------------------------------------------------------------------