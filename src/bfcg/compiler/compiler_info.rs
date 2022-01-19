use std::collections::HashMap;

use crate::bfcg::vm::port::Port;

use super::{dif_part_helper::{settings::Setting, setting_action_result::SettingActionResult}, mem_init::MemInit, compiler_warning::{CompilerWarnings, CompilerWarning}, comand_compiler::CmdCompiler, compiler_option::{CompilerOption, MemInitType}, compiler_pos::CompilerPos, compiler_error::IncludeError, compiler_inter_info::CompilerInterInfo};

#[derive(Debug)]
pub struct CompilerInfo<T>{
    mem_init: MemInit,
    program: Vec<T>,
    port_names: HashMap<String, usize>,
    devs: HashMap<Port, String>, // map port to dev name
    macros: HashMap<String, String>, // map name of macros to code

    warnings: CompilerWarnings,
    settings_for_parent: Vec<Setting>,

    inter_info: CompilerInterInfo,
}

impl<T> CompilerInfo<T>{
    pub fn new(inter_info: Option<CompilerInterInfo>) -> Self { 
        Self {
            mem_init: MemInit::new(),
            program: vec![],
            port_names: HashMap::new(), 
            devs: HashMap::new(),
            macros: HashMap::new(),
            warnings: CompilerWarnings::new(),
            settings_for_parent: vec![],
            inter_info: if let Some(inter_info) = inter_info { inter_info } else { CompilerInterInfo::new() },
        }
    }

    pub fn get_inter_info(&self) -> &CompilerInterInfo { &self.inter_info }
    pub fn get_mut_inter_info(&mut self) -> &mut CompilerInterInfo { &mut self.inter_info }

    pub fn get_ref_program(&self) -> &Vec<T> { &self.program }
    pub fn set_program(&mut self, program: Vec<T>) { 
        if !self.program.is_empty() { panic!("program already setted") }
        self.program = program; 
    }

    pub fn add_warning(&mut self, warning: CompilerWarning) { self.warnings.add_warning(warning) }
    pub fn add_warning_file_name_if_need(&mut self, file_name: String) {
        if !self.warnings.is_empty() { self.warnings.set_file(file_name) }
    }

    pub fn add_setting_for_parent(&mut self, setting: Setting) { self.settings_for_parent.push(setting) }

    pub fn clear_port_names(&mut self) { self.port_names.clear() }
    pub fn get_port(&self, port_name: &str) -> Option<usize> { self.port_names.get(port_name).cloned() }

    fn add_port_need_warning(&self, port_name: &str, port: usize) -> bool {
        if let Some(old_port) = self.port_names.get(port_name) {
            !(*old_port == port)
        } else { false }
    }
    /// ## Result
    /// if already exist port for this name => return Some(prev_value)
    /// else => return None
    pub fn add_port(&mut self, port_name: String, port: usize) -> Option<usize> { 
        if let Some(old_port) = self.port_names.insert(port_name, port) {
            if old_port == port { None }
            else { Some(old_port) }
        } else { None }
    }

    fn add_dev_need_warning(&self, port: &Port, dev_name: &str) -> bool {
        if let Some(old_dev_name) = self.devs.get(port) {
            old_dev_name != dev_name
        } else { false }
    }
    pub fn add_dev(&mut self, port: Port, dev_name: String) -> Option<String> { self.devs.insert(port, dev_name) }
    pub fn get_devs(&self) -> &HashMap<Port, String> { &self.devs }
    pub fn get_mut_devs(&mut self) -> &mut HashMap<Port, String> { &mut self.devs }

    pub fn get_mem_init(&self) -> &MemInit { &self.mem_init }
    pub fn get_mut_mem_init(&mut self) -> &mut MemInit { &mut self.mem_init }

    pub fn is_empty_mem_init(&self) -> bool { self.mem_init.is_empty() }

    /// general form of macro: #macro_name#macro_code#
    /// 
    /// general form of macro call: %macro_name%
    /// 
    /// this function take macro code and when in it 
    /// other macro call substitute instead name their code
    /// 
    /// ## Result
    /// * if all name known => Ok(macro_code (without inner calls))
    /// * else => Err(name of unknown macros)
    pub fn macro_transform(&self, macro_init_code: String) -> Result<String, String> {
        let mut macro_code = String::new();

        let macro_splited: Vec<&str> = macro_init_code.split('%').collect();
        let mut part_is_macro_name = false;

        for code_part in macro_splited {
            if !part_is_macro_name {
                macro_code += code_part;
            } else {
                let name = code_part;
                let code_part = self.get_macros_code(name);
                if code_part.is_none() { return Err(name.to_owned()) } 
                macro_code += code_part.unwrap();
            }
            part_is_macro_name = !part_is_macro_name;
        }

        Ok(macro_code)
    }

    /// ## Result
    /// * if cant self.add_macro => false
    /// * else => true 
    fn can_add_macro(&self, macro_name: &str) -> bool { !self.macros.contains_key(macro_name) }

    pub fn add_macro(&mut self, macro_name: String, macro_cmds: String) -> bool {
        if self.macros.contains_key(&macro_name) { false }
        else {  
            self.macros.insert(macro_name, macro_cmds);
            true
        }
    }

    /// ## result:
    /// * None => no error
    /// * Some(setting_string, ErrorResult)
    pub fn include_file_setting<CC>(&mut self, include: &mut Self, option: &CompilerOption<CC, T>, pos: CompilerPos) 
    -> Option<(String, SettingActionResult)> 
    where CC: CmdCompiler<T>
    {
        for setting in std::mem::take(&mut include.settings_for_parent) {
            let sa_res = option.setting_action.make_setting_action(&setting, self);

            if !sa_res.is_right_rule() { return Some((setting.to_string(), sa_res)) }

            let parent_must_process = sa_res.parent_must_process(); // is it always true?
            
            if let Some(warnings) = sa_res.get_warinings() {
                for warning in warnings {
                    self.warnings.add_warning(
                        CompilerWarning::SettingWarning{pos: pos.clone(), setting: setting.to_string(), warning}
                    );
                }
            }
            
            if parent_must_process { self.add_setting_for_parent(setting) }
        }

        None
    }

    pub fn add_include_file(&mut self, include: Self, pos: CompilerPos) -> Option<IncludeError> {
        if !include.program.is_empty() { panic!("program cant be non empty") }
        if !include.warnings.is_empty() {
            self.warnings.add_warning(CompilerWarning::FromOtherFile{ pos: pos.clone(), warnings: include.warnings })
        }

        if !include.settings_for_parent.is_empty() { panic!("you must previosly call include_file_setting"); }

        // update inter info (currently do {nothing, panic if logical error}):
        let include_inter_info = include.inter_info;
        self.get_mut_inter_info().update_with_other(&include_inter_info); 

        for (macro_name, macro_cmds) in include.macros {
            if !self.can_add_macro(&macro_name) { return Some(IncludeError::MacrosAlreadyDefined{ macro_name }) }
            if !self.add_macro(macro_name, macro_cmds) { panic!("cant be here") }
        }

        if let Some(mmc) = self.mem_init.merge(include.mem_init) {
            return Some(IncludeError::MemInitMergeError{ mmc })
        }

        for (port_name, port) in include.port_names {
            if self.add_port_need_warning(&port_name, port) {
                if let Some(old_port) = self.add_port(port_name.clone(), port) {
                    self.warnings.add_warning(
                        CompilerWarning::OtherPortUsedInOtherFile{ pos: pos.clone(), port_name, new_port: port, old_port }
                    );
                } else { panic!("warning dont need, but you say... you say that... that... you lied to me! ~baka~~") }
            } else {
                if let Some(_) = self.add_port(port_name, port) { panic!("there need warning, but you say that dont! ~baaaka~~") }    
            }
        }

        for (port, dev_name) in include.devs {
            if self.add_dev_need_warning(&port, &dev_name) {
                if let Some(x) = self.add_dev(port.clone(), dev_name.clone()) {
                    self.warnings.add_warning(
                        CompilerWarning::OtherDevUsedInOtherFile{ pos: pos.clone(), port, new_dev_name: dev_name, old_dev_name: x }
                    );
                } else { panic!("[tsundere panic] [must never happen] are you an idiot? can you just tell me if there will be an error or not?") }
            } else { self.add_dev(port, dev_name); }
        }

        None
    }

    pub fn get_macros_code(&self, macro_name: &str) -> Option<&str> {
        if let Some(code) = self.macros.get(macro_name) { Some(code) }
        else { None }
    }

    pub fn set_code_start(&mut self, mem_init_type: MemInitType) -> Option<String> {
        if !self.get_inter_info().is_code_started() {
            self.get_mut_inter_info().set_code_start();
            if mem_init_type.mem_init_only_before_code() {
                if self.mem_init.is_empty() { return None }
                return Some(self.mem_init.code_gen())
            }
        } 
        None 
    }
}
