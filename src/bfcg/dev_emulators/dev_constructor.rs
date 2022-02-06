use std::collections::{HashMap, HashSet};

use super::dev::Dev;


// TODO: string name ---> Dev 




pub enum DevCtorErr {
    UnknownName(String),
    BadParamValue{ name: String, bad_value: String },
    Other(String),
}

pub enum DevCtorWarn {
    UnusedDevParam(String),
    Other(String),
}

pub trait DevCtor {
    fn dev_ctor(dev_name_params: &HashMap<String, String>) -> Result<DevCtorOk, DevCtorErr>;
}

pub struct DevCtorOk{
    pub dev: Box<dyn Dev>,
    pub warns: Vec<DevCtorWarn>,
}

impl DevCtorOk{
    pub fn new(dev: Box<dyn Dev>, warns:Vec<DevCtorWarn>) -> Self { Self { dev, warns } }
}

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
macro_rules! dev_ctor_no_param_impl {
    ($dev_type: ty) => {
        impl crate::bfcg::dev_emulators::dev_constructor::DevCtor for $dev_type {
            fn dev_ctor(dev_name_params: &std::collections::HashMap<String, String>) 
            -> Result<crate::bfcg::dev_emulators::dev_constructor::DevCtorOk, 
                      crate::bfcg::dev_emulators::dev_constructor::DevCtorErr> 
            {
                let mut helper = crate::bfcg::dev_emulators::dev_constructor::DevCtorHelper::new(dev_name_params);

                helper.add_unused_warn();
                let warns = helper.take_warn();
        
                Ok(crate::bfcg::dev_emulators::dev_constructor::DevCtorOk::new(Box::new(<$dev_type>::new()), warns))
            }
        }
    };
}


#[macro_export]
macro_rules! dev_ctor_one_param_impl {
    ($dev_type: ty, $await_name:expr, $default:ident) => {
        impl crate::bfcg::dev_emulators::dev_constructor::DevCtor for $dev_type {
            fn dev_ctor(dev_name_params: &std::collections::HashMap<String, String>) 
            -> Result<crate::bfcg::dev_emulators::dev_constructor::DevCtorOk, 
                      crate::bfcg::dev_emulators::dev_constructor::DevCtorErr> 
            {
                let mut helper = crate::bfcg::dev_emulators::dev_constructor::DevCtorHelper::new(dev_name_params);

                let x = crate::dev_ctor_parse_unwrap!(helper, $await_name, $default);
                helper.add_unused_warn();
                let warns = helper.take_warn();
        
                Ok(crate::bfcg::dev_emulators::dev_constructor::DevCtorOk::new(Box::new(<$dev_type>::new(x)), warns))
            }
        }
    };
}

// [-] MACROS
//------------------------------------------------------------------------