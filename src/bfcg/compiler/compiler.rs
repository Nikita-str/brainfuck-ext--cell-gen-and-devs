use std::collections::HashMap;
use std::str::Chars;

use crate::bfcg::compiler::dif_part_helper::settings::Setting;
use crate::bfcg::vm::port::Port;

use super::cmd_compiler::CmdCompiler;
use super::compiler_error::{CompilerError};
use super::compiler_option::CompilerOption;
use super::compiler_pos::CompilerPos;
use super::mem_init::MemInit;

pub fn file_minimalize(str: &str) -> Result<String, ()>{
    let mut comment_until_next_line = false;
    let mut err = None;

    let ret = str.chars().filter(|c|{
        if err.is_some() { return false }
        match c {
            ';' => { 
                comment_until_next_line = true; 
                false
            }
            x if x.is_ascii_whitespace() => {
                if *x == '\n' { comment_until_next_line = false; }
                false
            }
            x if x.is_ascii_graphic() => {
                !comment_until_next_line
            }
            _ => { 
                if !comment_until_next_line { err = Some(()); } 
                false 
            }
        }
    }).collect();

    if err.is_some() { return Err(err.unwrap()) }

    Ok(ret)
}

struct InnerCompilerParam<'a>{
    chars: Chars<'a>,
    pos: CompilerPos,
}

impl<'a> InnerCompilerParam<'a>{
    pub fn new(chars: Chars<'a>) -> Self{ 
        Self {
            chars,
            pos: CompilerPos::new(),
        }
    }

    pub fn next(&mut self) -> Option<char> {
        let c = self.chars.next();
        self.pos.maybe_add_char(c);
        c
    }

    pub fn get_pos(&self) -> CompilerPos { self.pos.clone() }
}

#[derive(Debug)]
pub struct CompilerInfo<T>{
    mem_init: MemInit,
    program: Vec<T>,
    port_names: HashMap<String, usize>,
    devs: HashMap<Port, String>, // map port to dev name
    macros: HashMap<String, String>, // map name of macros to code
}

impl<T> CompilerInfo<T>{
    pub fn new() -> Self { 
        Self {
            mem_init: MemInit::new(),
            program: vec![],
            port_names: HashMap::new(), 
            devs: HashMap::new(),
            macros: HashMap::new(),
        }
    }

    pub fn clear_port_names(&mut self) { self.port_names.clear() }
    pub fn get_port(&self, port_name: &str) -> Option<usize> { self.port_names.get(port_name).cloned() }

    /// ## Result
    /// if already exist port for this name => return Some(prev_value)
    /// else => return None
    pub fn add_port(&mut self, port_name: String, port: usize) -> Option<usize> { self.port_names.insert(port_name, port) }

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

    pub fn add_macro(&mut self, macro_name: String, macro_cmds: String) -> bool {
        if self.macros.contains_key(&macro_name) { false }
        else {  
            self.macros.insert(macro_name, macro_cmds);
            true
        }
    }

    pub fn add_include_file(&mut self, include: Self) -> bool {
        for (macro_name, macro_cmds) in include.macros {
            if !self.add_macro(macro_name, macro_cmds) { return false }
        }
        true
    }

    pub fn get_macros_code(&self, macro_name: &str) -> Option<&str> {
        if let Some(code) = self.macros.get(macro_name) { Some(code) }
        else { None }
    }
}

pub(super) const MACRO_USE_CHAR: char = '%';
pub(super) const SETTINGS_CHAR: char = '\'';
pub(super) const COMMENT_LINE: char = ';';
pub(super) const NEXT_LINE: char = '\n';
pub(super) const SHARP: char = '#';

fn skip_line(param: &mut InnerCompilerParam){
    loop {
        match param.next() {
            Some(super::compiler::NEXT_LINE) | None => break,
            _ => {}
        }
    }
}

fn parse_until_char(param: &mut InnerCompilerParam, first_char: Option<char>, until_char: char) -> Option<String>{
    let mut ret_str = String::new();
    if let Some(x) = first_char { ret_str.push(x); }
    loop {
        match param.next() {
            None => { return None }
            Some(super::compiler::COMMENT_LINE) => skip_line(param),
            Some(c) if c == until_char => { break },
            Some(c) if c.is_whitespace() => { },
            Some(c) => { ret_str.push(c); },
        };
    }
    
    Some(ret_str)
}

enum SharpParse{
    UnexpectedEOF,
    EmptyName,
    BadMacroName(char),
    FileInclude(String),
    Macros{macro_name: String, macro_cmds: String},

    Temp(String),
} 

impl SharpParse{
    fn is_temp(&self) -> bool { if let Self::Temp(_) = self { true } else { false } }
    fn temp_to_string(self) -> Option<String> { if let Self::Temp(x) = self { Some(x) } else { None } } 

    fn to_file_include(self) -> Self { 
        match self {
            Self::BadMacroName(_) | Self::UnexpectedEOF | Self::EmptyName | Self::FileInclude(_) => self,
            Self::Temp(s) => Self::FileInclude(s),
            Self::Macros{ .. } => panic!("macro can't transform into include"), 
        }
    }

    fn parse_until_sharp(param: &mut InnerCompilerParam, first_char: Option<char>) -> SharpParse{
        let ret_str = parse_until_char(param, first_char, super::compiler::SHARP);
        if let None = ret_str { return SharpParse::UnexpectedEOF }

        let ret_str = ret_str.unwrap();
        if ret_str.len() == 0 { SharpParse::EmptyName }
        else { SharpParse::Temp(ret_str) }
    }

    fn parse_sharp(param: &mut InnerCompilerParam) -> SharpParse{
        let c;
        loop {
            match param.next() {
                None => { return SharpParse::UnexpectedEOF }
                Some(super::compiler::COMMENT_LINE) => { skip_line(param) },
                Some(super::compiler::MACRO_USE_CHAR) => { return SharpParse::BadMacroName(super::compiler::MACRO_USE_CHAR) },
                Some(c) if c.is_whitespace() => { },
                Some(ok) => { c = Some(ok); break },
            };
        }
        let c = c.unwrap(); // must always ok; but if in loop exist algo error => exception

        if c == super::compiler::SHARP { return SharpParse::to_file_include(Self::parse_until_sharp(param, None)) }
            
        let macro_name = Self::parse_until_sharp(param, Some(c));
        let macro_name =
        if !macro_name.is_temp() { return macro_name } 
        else { macro_name.temp_to_string().unwrap() };
        if macro_name.contains(super::compiler::MACRO_USE_CHAR) { 
            return SharpParse::BadMacroName(super::compiler::MACRO_USE_CHAR) 
        }

        let macro_cmds = Self::parse_until_sharp(param, None);
        let macro_cmds =
        if !macro_cmds.is_temp() { return macro_cmds } 
        else { macro_cmds.temp_to_string().unwrap() };

        SharpParse::Macros{ macro_name, macro_cmds }
    }
}

type CE = CompilerError;

// TODO: type of error insted of (); if return None => no errors
pub fn compile<CC, T>(file_name: String, mut option: CompilerOption<CC, T>) -> Result<CompilerInfo<T>, CompilerError>
where CC: CmdCompiler<T>,
{
    let file_as_string = std::fs::read_to_string(&file_name);
    if file_as_string.is_err() { return Err(CompilerError::new_file_open_err(file_name)) }
    let file_as_string = file_as_string.unwrap();
    let chars = file_as_string.chars();

    let mut ret = CompilerInfo::new();
    let mut param = InnerCompilerParam::new(chars);

    loop {
        match param.next() {
            None => {
                if !option.only_macros {
                    let program = option.cmd_compiler.unwrap().get_program();
                    if let Err(err) = program { return Err(err) } //TODO: file_name!
                    else { ret.program = program.ok().unwrap(); }
                }
                return Ok(ret)
            }
            Some(super::compiler::COMMENT_LINE) => { 
                skip_line(&mut param); 
            }

            Some(c) if c.is_whitespace() => { }

            Some(super::compiler::SHARP) => { 
                match SharpParse::parse_sharp(&mut param) {
                    SharpParse::EmptyName => return Err(CE::new_empty_name(param.get_pos(), file_name)),
                    SharpParse::UnexpectedEOF => return Err(CE::new_unexp_eof(param.get_pos(), file_name)),
                    SharpParse::BadMacroName(bad_char) => return Err(CE::new_bad_macro_name(param.get_pos(), file_name, bad_char)),
                    SharpParse::Temp(_) => panic!("here must not be temp"),
                    SharpParse::FileInclude(include) => {
                        let include_compile = compile(include, CompilerOption::<CC, T>::new_only_macro());
                        let include_compile = if let Err(mut err) = include_compile { 
                            err.add_err_pos(param.get_pos(), file_name);
                            return Err(err)
                        } else {
                            include_compile.ok().unwrap()
                        };
                        if !ret.add_include_file(include_compile) {
                            return Err(CE::new_already_defined(param.get_pos(), file_name))
                        }
                    }
                    SharpParse::Macros{ macro_name, macro_cmds } => {
                        let macro_code = ret.macro_transform(macro_cmds);
                        let macro_code = if let Ok(macro_code) = macro_code { macro_code }
                        else { return Err(CE::new_unknown_macros(param.get_pos(), file_name, macro_code.err().unwrap())) };

                        if !ret.add_macro(macro_name, macro_code) {
                            return Err(CE::new_already_defined(param.get_pos(), file_name))
                        }
                    }
                }
            }

            Some(_) if option.only_macros => {
                return Err(CE::new_code_in_macros(param.get_pos(), file_name)) 
            }

            Some(super::compiler::SETTINGS_CHAR) => {
                let setting = parse_until_char(&mut param, None, super::compiler::SETTINGS_CHAR);
                let setting = 
                    if let Some(setting) = setting { setting } 
                    else { return Err(CE::new_unexp_eof(param.get_pos(), file_name)) };
                
                match Setting::prepare_settings(&setting) {
                    Err(error) => return Err(CE::new_setting_error(param.get_pos(), file_name, error)),
                    Ok(setting) => {
                        todo!("todo")
                    }
                }

                todo!("todo")
            }

            Some(super::compiler::MACRO_USE_CHAR) => {
                let macros_name = parse_until_char(&mut param, None, super::compiler::MACRO_USE_CHAR);
                if let Some(macros_name) = macros_name {
                    let macros_code = ret.get_macros_code(&macros_name);
                    if macros_code.is_none() { 
                        return Err(CE::new_unknown_macros(param.get_pos(), file_name, macros_name)) 
                    } else {
                        let macros_code = macros_code.unwrap().chars();
                        let cc = option.cmd_compiler.as_mut().unwrap();
                        for c in macros_code {
                            if let Some(mut err) = cc.cmd_compile(c, param.get_pos()){
                                err.add_err_pos(param.get_pos(), file_name);
                                return Err(err)
                            }                            
                        }
                    }
                } else {
                    return Err(CE::new_unexp_eof(param.get_pos(), file_name))
                }
            }

            Some(c) => {
                let cc = option.cmd_compiler.as_mut().unwrap();
                if let Some(mut err) = cc.cmd_compile(c, param.get_pos()){
                    err.add_err_pos(param.get_pos(), file_name);
                    return Err(err)
                }
            }
        }
    }
}
